use garuda::chess::{
    evaluate_nes_fitness, run_nes_step, Engine, GameStatus, MctsConfig, MctsEngine, NesConfig,
    Position, SearchConfig, TinyNeuralModel,
};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  garuda-chess bestmove [fen] [garuda_depth] [garuda_quiescence]");
    eprintln!("  garuda-chess bestmove-mcts [fen] [simulations] [cpuct]");
    eprintln!("  garuda-chess bestmove-mcts-vector <fen> <vector_file> [simulations] [cpuct]");
    eprintln!("  garuda-chess model-vector");
    eprintln!("  garuda-chess nes-eval [vector_file] [simulations] [cpuct]");
    eprintln!("  garuda-chess nes-step <output_vector_file> [input_vector_file] [population_size] [sigma] [learning_rate] [seed] [simulations] [cpuct]");
    eprintln!("  garuda-chess nes-train <output_vector_file> [input_vector_file] [generations] [population_size] [sigma] [learning_rate] [seed] [simulations] [cpuct]");
    eprintln!("  garuda-chess apply <fen> <uci>");
    eprintln!("  garuda-chess status [fen]");
    eprintln!("  garuda-chess match-uci <engine_command> [max_plies_or_0] [movetime_ms] [garuda_color] [garuda_depth] [garuda_quiescence]");
    eprintln!("  garuda-chess bo-uci <engine_command> [games] [max_plies_or_0] [movetime_ms] [garuda_depth] [garuda_quiescence] [openings_file]");
    eprintln!("  garuda-chess match-uci-mcts <engine_command> [max_plies_or_0] [movetime_ms] [garuda_color] [simulations] [cpuct]");
    eprintln!("  garuda-chess match-uci-mcts-vector <engine_command> <vector_file> [max_plies_or_0] [movetime_ms] [garuda_color] [simulations] [cpuct]");
    eprintln!("  garuda-chess bo-uci-mcts <engine_command> [games] [max_plies_or_0] [movetime_ms] [simulations] [cpuct] [openings_file]");
    eprintln!("  garuda-chess bo-uci-mcts-vector <engine_command> <vector_file> [games] [max_plies_or_0] [movetime_ms] [simulations] [cpuct] [openings_file]");
    eprintln!("  garuda-chess nes-eval-uci <engine_command> [vector_file] [games] [max_plies_or_0] [movetime_ms] [simulations] [cpuct] [openings_file]");
    eprintln!("  garuda-chess nes-train-uci <engine_command> <output_vector_file> [input_vector_file] [generations] [population_size] [sigma] [learning_rate] [seed] [games] [max_plies_or_0] [movetime_ms] [simulations] [cpuct] [openings_file] [run_dir]");
}

fn vector_to_line(vector: &[f32]) -> String {
    vector
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

fn write_vector_file(path: &str, vector: &[f32]) -> Result<(), String> {
    fs::write(path, format!("{}\n", vector_to_line(vector)))
        .map_err(|error| format!("failed to write vector file {path}: {error}"))
}

fn append_file_line(path: &str, line: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open log file {path}: {error}"))?;
    writeln!(file, "{line}").map_err(|error| format!("failed to append log file {path}: {error}"))
}

fn write_nes_train_run_config(
    run_dir: &str,
    engine_command: &str,
    output_vector_file: &str,
    input_vector_file: Option<&str>,
    generations: usize,
    population_size: usize,
    sigma: f32,
    learning_rate: f32,
    seed: u64,
    games: usize,
    max_plies: Option<usize>,
    movetime_ms: u64,
    simulations: usize,
    cpuct: f32,
    openings_count: usize,
) -> Result<(), String> {
    fs::create_dir_all(run_dir)
        .map_err(|error| format!("failed to create run directory {run_dir}: {error}"))?;
    let max_plies_value = max_plies
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    let input_vector_value = input_vector_file.unwrap_or("<default-model>");
    let config_path = format!("{run_dir}/config.txt");
    let config = format!(
        "engine_command={engine_command}\noutput_vector_file={output_vector_file}\ninput_vector_file={input_vector_value}\ngenerations={generations}\npopulation_size={population_size}\nsigma={sigma}\nlearning_rate={learning_rate}\nseed={seed}\ngames={games}\nmax_plies={max_plies_value}\nmovetime_ms={movetime_ms}\nsimulations={simulations}\ncpuct={cpuct}\nopenings_count={openings_count}\n"
    );
    fs::write(&config_path, config)
        .map_err(|error| format!("failed to write run config {config_path}: {error}"))?;
    let progress_path = format!("{run_dir}/progress.tsv");
    fs::write(
        &progress_path,
        "generation\tinitial_fitness\tbest_candidate_fitness\tupdated_fitness\ttotal_evaluations\tvector_file\n",
    )
    .map_err(|error| format!("failed to write progress header {progress_path}: {error}"))
}

fn write_uci_game_records(path: &str, records: &[UciFitnessGameRecord]) -> Result<(), String> {
    let mut contents =
        "game_index\tgaruda_color\tresult\tstatus\tstart_fen\tfinal_fen\tplies\n".to_string();
    for record in records {
        contents.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            record.game_index,
            record.garuda_color,
            record.result,
            record.status,
            record.start_fen,
            record.final_fen,
            record.plies
        ));
    }
    fs::write(path, contents)
        .map_err(|error| format!("failed to write uci game records {path}: {error}"))
}

struct UciEngine {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl UciEngine {
    fn spawn(engine_command: &str) -> Result<Self, String> {
        let mut child = if engine_command.contains(' ') {
            Command::new("sh")
                .arg("-lc")
                .arg(engine_command)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|error| format!("failed to launch engine command: {error}"))?
        } else {
            Command::new(engine_command)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|error| format!("failed to launch engine: {error}"))?
        };
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "missing engine stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "missing engine stdout".to_string())?;
        let mut engine = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        };
        engine.command("uci")?;
        engine.read_until("uciok")?;
        engine.command("isready")?;
        engine.read_until("readyok")?;
        Ok(engine)
    }

    fn command(&mut self, line: &str) -> Result<(), String> {
        writeln!(self.stdin, "{line}")
            .map_err(|error| format!("failed to write to engine: {error}"))?;
        self.stdin
            .flush()
            .map_err(|error| format!("failed to flush engine stdin: {error}"))?;
        Ok(())
    }

    fn read_until(&mut self, needle: &str) -> Result<String, String> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = self
                .stdout
                .read_line(&mut line)
                .map_err(|error| format!("failed to read engine output: {error}"))?;
            if bytes == 0 {
                return Err("engine closed stdout unexpectedly".to_string());
            }
            if line.trim_start().starts_with(needle) {
                return Ok(line.trim().to_string());
            }
        }
    }

    fn bestmove(&mut self, position: &Position, movetime_ms: u64) -> Result<String, String> {
        self.command("ucinewgame")?;
        self.command("isready")?;
        self.read_until("readyok")?;
        self.command(&format!("position fen {}", position.to_fen()))?;
        self.command(&format!("go movetime {movetime_ms}"))?;
        let bestmove_line = self.read_until("bestmove")?;
        let mut parts = bestmove_line.split_whitespace();
        let _ = parts.next();
        parts
            .next()
            .map(str::to_string)
            .ok_or_else(|| "engine did not return a bestmove".to_string())
    }

    fn shutdown(&mut self) {
        let _ = self.command("quit");
        let _ = self.child.wait();
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn parse_color(text: &str) -> Result<garuda::chess::Color, String> {
    match text {
        "w" | "white" => Ok(garuda::chess::Color::White),
        "b" | "black" => Ok(garuda::chess::Color::Black),
        _ => Err(format!("invalid color: {text}")),
    }
}

fn format_status(status: GameStatus) -> &'static str {
    match status {
        GameStatus::Ongoing => "ongoing",
        GameStatus::Checkmate { loser } => match loser {
            garuda::chess::Color::White => "checkmate:white-loses",
            garuda::chess::Color::Black => "checkmate:black-loses",
        },
        GameStatus::Stalemate { side_to_move } => match side_to_move {
            garuda::chess::Color::White => "stalemate:white-to-move",
            garuda::chess::Color::Black => "stalemate:black-to-move",
        },
        GameStatus::DrawByFiftyMoveRule => "draw:fifty-move-rule",
        GameStatus::DrawByRepetition => "draw:threefold-repetition",
    }
}

fn parse_usize_arg(value: Option<String>, default: usize) -> usize {
    value
        .and_then(|text| text.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_u64_arg(value: Option<String>, default: u64) -> u64 {
    value
        .and_then(|text| text.parse::<u64>().ok())
        .unwrap_or(default)
}

fn parse_optional_plies_arg(value: Option<String>, default: Option<usize>) -> Option<usize> {
    match value {
        Some(text) => match text.parse::<usize>().ok() {
            Some(0) => None,
            Some(plies) => Some(plies),
            None => default,
        },
        None => default,
    }
}

fn build_search_config(depth: usize, quiescence_depth: usize) -> SearchConfig {
    SearchConfig {
        max_depth: depth,
        quiescence_depth,
        ..SearchConfig::default()
    }
}

fn build_mcts_config(simulations: usize, cpuct: f32) -> MctsConfig {
    MctsConfig {
        simulations,
        cpuct,
        ..MctsConfig::default()
    }
}

fn parse_f32_arg(value: Option<String>, default: f32) -> f32 {
    value
        .and_then(|text| text.parse::<f32>().ok())
        .unwrap_or(default)
}

fn load_openings(path: &str) -> Result<Vec<Position>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read openings file: {error}"))?;
    let mut openings = Vec::new();
    for (line_index, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let position = Position::from_fen(trimmed)
            .map_err(|error| format!("invalid FEN on line {}: {error}", line_index + 1))?;
        openings.push(position);
    }
    if openings.is_empty() {
        return Err("openings file contained no usable FEN positions".to_string());
    }
    Ok(openings)
}

fn load_parameter_vector(path: &str) -> Result<Vec<f32>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read parameter vector: {error}"))?;
    let normalized = contents.replace(',', " ");
    let mut values = Vec::new();
    for token in normalized.split_whitespace() {
        let value = token
            .parse::<f32>()
            .map_err(|error| format!("invalid float in parameter vector: {error}"))?;
        values.push(value);
    }
    if values.is_empty() {
        return Err("parameter vector file was empty".to_string());
    }
    Ok(values)
}

fn load_model_from_vector_file(path: &str) -> Result<TinyNeuralModel, String> {
    let vector = load_parameter_vector(path)?;
    TinyNeuralModel::from_parameter_vector(&vector)
}

fn chrono_like_utc_date() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let days = seconds / 86_400;
    civil_from_days(days as i64)
}

fn civil_from_days(days_since_epoch: i64) -> String {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    format!("{year:04}.{month:02}.{day:02}")
}

fn format_pgn_result(status: GameStatus) -> &'static str {
    match status {
        GameStatus::Checkmate {
            loser: garuda::chess::Color::White,
        } => "0-1",
        GameStatus::Checkmate {
            loser: garuda::chess::Color::Black,
        } => "1-0",
        GameStatus::Stalemate { .. }
        | GameStatus::DrawByFiftyMoveRule
        | GameStatus::DrawByRepetition
        | GameStatus::Ongoing => "1/2-1/2",
    }
}

fn format_pgn_moves(san_moves: &[String], result: &str) -> String {
    let mut tokens = Vec::new();
    for (index, san) in san_moves.iter().enumerate() {
        if index % 2 == 0 {
            tokens.push(format!("{}. {}", (index / 2) + 1, san));
        } else {
            tokens.push(san.clone());
        }
    }
    tokens.push(result.to_string());
    tokens.join(" ")
}

fn play_match_game<F>(
    choose_move: F,
    uci: &mut UciEngine,
    max_plies: Option<usize>,
    movetime_ms: u64,
    garuda_color: garuda::chess::Color,
    emit_moves: bool,
    start_position: &Position,
) -> Result<(Position, GameStatus, Vec<String>), String>
where
    F: Fn(&Position, &[u64]) -> Option<garuda::chess::ChessMove>,
{
    let mut position = start_position.clone();
    let mut repetition_history = vec![position.repetition_key()];
    let mut san_moves = Vec::new();
    let mut ply = 0usize;
    loop {
        if let Some(max_plies) = max_plies {
            if ply >= max_plies {
                break;
            }
        }
        let status = position.game_status_with_history(&repetition_history);
        if status != GameStatus::Ongoing {
            break;
        }

        let side = position.side_to_move();
        let uci_move = if side == garuda_color {
            match choose_move(&position, &repetition_history) {
                Some(chess_move) => chess_move.uci(),
                None => break,
            }
        } else {
            uci.bestmove(&position, movetime_ms)?
        };

        if emit_moves {
            println!("ply {} {} {}", ply + 1, side.fen_symbol(), uci_move);
        }
        let chess_move = position
            .legal_moves()
            .into_iter()
            .find(|candidate| candidate.uci() == uci_move)
            .ok_or_else(|| format!("illegal move from engine: {uci_move}"))?;
        let san = position
            .san_for_move(&chess_move)
            .ok_or_else(|| format!("failed to convert move to SAN: {uci_move}"))?;
        let Some(next) = position.apply_uci_move(&uci_move) else {
            return Err(format!("illegal move from engine: {uci_move}"));
        };
        san_moves.push(san);
        position = next;
        repetition_history.push(position.repetition_key());
        ply += 1;
    }
    let final_status = position.game_status_with_history(&repetition_history);
    Ok((position, final_status, san_moves))
}

fn print_match_pgn(
    event_name: &str,
    position: &Position,
    status: GameStatus,
    san_moves: &[String],
    garuda_color: garuda::chess::Color,
    garuda_name: &str,
) {
    println!("result {}", format_status(status));
    println!("final_fen {}", position.to_fen());
    let pgn_result = format_pgn_result(status);
    println!("[Event \"{event_name}\"]");
    println!("[Site \"?\"]");
    println!("[Date \"{}\"]", chrono_like_utc_date());
    println!("[Round \"?\"]");
    println!(
        "[White \"{}\"]",
        if garuda_color == garuda::chess::Color::White {
            garuda_name
        } else {
            "Stockfish"
        }
    );
    println!(
        "[Black \"{}\"]",
        if garuda_color == garuda::chess::Color::Black {
            garuda_name
        } else {
            "Stockfish"
        }
    );
    println!("[Result \"{}\"]", pgn_result);
    println!();
    println!("{}", format_pgn_moves(san_moves, pgn_result));
}

fn summarize_bo_result(
    status: GameStatus,
    garuda_color: garuda::chess::Color,
    garuda_wins: &mut usize,
    uci_wins: &mut usize,
    draws: &mut usize,
) -> &'static str {
    match status {
        GameStatus::Checkmate { loser } if loser == garuda_color => {
            *uci_wins += 1;
            "uci-win"
        }
        GameStatus::Checkmate { .. } => {
            *garuda_wins += 1;
            "garuda-win"
        }
        GameStatus::Stalemate { .. }
        | GameStatus::DrawByFiftyMoveRule
        | GameStatus::DrawByRepetition
        | GameStatus::Ongoing => {
            *draws += 1;
            "draw"
        }
    }
}

#[derive(Clone)]
struct UciFitnessConfig {
    games: usize,
    max_plies: Option<usize>,
    movetime_ms: u64,
    simulations: usize,
    cpuct: f32,
    openings: Vec<Position>,
}

#[derive(Debug, Clone)]
struct UciFitnessResult {
    fitness: f32,
    garuda_wins: usize,
    uci_wins: usize,
    draws: usize,
    games: Vec<UciFitnessGameRecord>,
}

#[derive(Debug, Clone)]
struct UciFitnessGameRecord {
    game_index: usize,
    garuda_color: char,
    result: String,
    status: String,
    start_fen: String,
    final_fen: String,
    plies: usize,
}

#[derive(Debug, Clone)]
struct CliXorShift64 {
    state: u64,
}

impl CliXorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn next_f32(&mut self) -> f32 {
        let value = self.next_u64() >> 40;
        (value as f32) / ((1u32 << 24) as f32)
    }

    fn standard_normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1.0e-7);
        let u2 = self.next_f32();
        let radius = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f32::consts::PI * u2;
        radius * theta.cos()
    }
}

fn evaluate_mcts_model_against_uci(
    model: TinyNeuralModel,
    engine_command: &str,
    config: &UciFitnessConfig,
) -> Result<UciFitnessResult, String> {
    let engine = MctsEngine::new(model, build_mcts_config(config.simulations, config.cpuct));
    let mut uci = UciEngine::spawn(engine_command)?;
    let mut garuda_wins = 0usize;
    let mut uci_wins = 0usize;
    let mut draws = 0usize;
    let mut games = Vec::new();

    for game_index in 0..config.games {
        let garuda_color = if game_index % 2 == 0 {
            garuda::chess::Color::White
        } else {
            garuda::chess::Color::Black
        };
        let start_position = &config.openings[game_index % config.openings.len()];
        let (position, status, san_moves) = play_match_game(
            |position, repetition_history| {
                engine.best_move_with_history(position, repetition_history)
            },
            &mut uci,
            config.max_plies,
            config.movetime_ms,
            garuda_color,
            false,
            start_position,
        )?;
        let result = summarize_bo_result(
            status,
            garuda_color,
            &mut garuda_wins,
            &mut uci_wins,
            &mut draws,
        );
        games.push(UciFitnessGameRecord {
            game_index: game_index + 1,
            garuda_color: garuda_color.fen_symbol(),
            result: result.to_string(),
            status: format_status(status).to_string(),
            start_fen: start_position.to_fen(),
            final_fen: position.to_fen(),
            plies: san_moves.len(),
        });
    }

    Ok(UciFitnessResult {
        fitness: garuda_wins as f32 + (draws as f32 * 0.5),
        garuda_wins,
        uci_wins,
        draws,
        games,
    })
}

fn run_nes_step_uci(
    base_vector: &[f32],
    nes_config: NesConfig,
    engine_command: &str,
    uci_config: &UciFitnessConfig,
) -> Result<(f32, f32, f32, usize, Vec<f32>), String> {
    let initial = evaluate_mcts_model_against_uci(
        TinyNeuralModel::from_parameter_vector(base_vector)?,
        engine_command,
        uci_config,
    )?
    .fitness;
    let pair_count = (nes_config.population_size.max(2) + 1) / 2;
    let sigma = nes_config.sigma.max(1.0e-4);
    let mut rng = CliXorShift64::new(nes_config.seed);
    let mut gradient = vec![0.0; base_vector.len()];
    let mut best_candidate_fitness = initial;
    let mut best_vector = base_vector.to_vec();
    let mut evaluations = 1usize;

    for _ in 0..pair_count {
        let epsilon: Vec<f32> = (0..base_vector.len())
            .map(|_| rng.standard_normal())
            .collect();
        let plus_vector: Vec<f32> = base_vector
            .iter()
            .zip(epsilon.iter())
            .map(|(weight, noise)| weight + (noise * sigma))
            .collect();
        let minus_vector: Vec<f32> = base_vector
            .iter()
            .zip(epsilon.iter())
            .map(|(weight, noise)| weight - (noise * sigma))
            .collect();
        let plus_fitness = evaluate_mcts_model_against_uci(
            TinyNeuralModel::from_parameter_vector(&plus_vector)?,
            engine_command,
            uci_config,
        )?
        .fitness;
        let minus_fitness = evaluate_mcts_model_against_uci(
            TinyNeuralModel::from_parameter_vector(&minus_vector)?,
            engine_command,
            uci_config,
        )?
        .fitness;
        evaluations += 2;

        if plus_fitness > best_candidate_fitness {
            best_candidate_fitness = plus_fitness;
            best_vector = plus_vector.clone();
        }
        if minus_fitness > best_candidate_fitness {
            best_candidate_fitness = minus_fitness;
            best_vector = minus_vector.clone();
        }

        let delta = plus_fitness - minus_fitness;
        for (index, noise) in epsilon.iter().enumerate() {
            gradient[index] += delta * noise;
        }
    }

    let gradient_scale = nes_config.learning_rate / (pair_count as f32 * sigma);
    let mut updated_vector: Vec<f32> = base_vector
        .iter()
        .zip(gradient.iter())
        .map(|(weight, grad)| weight + (gradient_scale * grad))
        .collect();
    let updated_fitness = evaluate_mcts_model_against_uci(
        TinyNeuralModel::from_parameter_vector(&updated_vector)?,
        engine_command,
        uci_config,
    )?
    .fitness;
    evaluations += 1;
    if best_candidate_fitness > updated_fitness {
        updated_vector = best_vector;
    }
    let final_fitness = evaluate_mcts_model_against_uci(
        TinyNeuralModel::from_parameter_vector(&updated_vector)?,
        engine_command,
        uci_config,
    )?
    .fitness;
    evaluations += 1;

    Ok((
        initial,
        best_candidate_fitness,
        final_fitness,
        evaluations,
        updated_vector,
    ))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(1);
    };

    match command.as_str() {
        "bestmove" => {
            let fen = args
                .next()
                .unwrap_or_else(|| Position::STARTPOS_FEN.to_string());
            let garuda_depth = parse_usize_arg(args.next(), SearchConfig::default().max_depth);
            let garuda_quiescence =
                parse_usize_arg(args.next(), SearchConfig::default().quiescence_depth);
            let position = match Position::from_fen(&fen) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("invalid fen: {error}");
                    std::process::exit(2);
                }
            };
            let engine = Engine::new(
                TinyNeuralModel::default(),
                build_search_config(garuda_depth, garuda_quiescence),
            );
            match engine.best_move(&position) {
                Some(chess_move) => println!("{}", chess_move.uci()),
                None => {
                    eprintln!("no move available");
                    std::process::exit(3);
                }
            }
        }
        "bestmove-mcts" => {
            let fen = args
                .next()
                .unwrap_or_else(|| Position::STARTPOS_FEN.to_string());
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let position = match Position::from_fen(&fen) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("invalid fen: {error}");
                    std::process::exit(2);
                }
            };
            let engine = MctsEngine::new(
                TinyNeuralModel::default(),
                build_mcts_config(simulations, cpuct),
            );
            match engine.best_move(&position) {
                Some(chess_move) => println!("{}", chess_move.uci()),
                None => {
                    eprintln!("no move available");
                    std::process::exit(3);
                }
            }
        }
        "bestmove-mcts-vector" => {
            let Some(fen) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let Some(vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let position = match Position::from_fen(&fen) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("invalid fen: {error}");
                    std::process::exit(2);
                }
            };
            let vector = match load_parameter_vector(&vector_file) {
                Ok(vector) => vector,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let model = match TinyNeuralModel::from_parameter_vector(&vector) {
                Ok(model) => model,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let engine = MctsEngine::new(model, build_mcts_config(simulations, cpuct));
            match engine.best_move(&position) {
                Some(chess_move) => println!("{}", chess_move.uci()),
                None => {
                    eprintln!("no move available");
                    std::process::exit(3);
                }
            }
        }
        "model-vector" => {
            let model = TinyNeuralModel::default();
            let vector = model.parameter_vector();
            let line = vector
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            println!("{line}");
        }
        "nes-eval" => {
            let vector = match args.next() {
                Some(vector_file) => match load_parameter_vector(&vector_file) {
                    Ok(vector) => vector,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => TinyNeuralModel::default().parameter_vector(),
            };
            let simulations = parse_usize_arg(args.next(), 32);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let evaluation =
                match evaluate_nes_fitness(&vector, build_mcts_config(simulations, cpuct)) {
                    Ok(evaluation) => evaluation,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                };
            println!("fitness {}", evaluation.total_fitness);
            for case in evaluation.cases {
                println!(
                    "case {} move={} preferred_hit={} avoided_hit={} preferred_policy_mass={} avoided_policy_mass={} value_alignment={} score={}",
                    case.name,
                    case.chosen_move.unwrap_or_else(|| "-".to_string()),
                    case.preferred_hit,
                    case.avoided_hit,
                    case.preferred_policy_mass,
                    case.avoided_policy_mass,
                    case.value_alignment,
                    case.score
                );
            }
        }
        "nes-step" => {
            let Some(output_vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let base_vector = match args.next() {
                Some(input_vector_file) => match load_parameter_vector(&input_vector_file) {
                    Ok(vector) => vector,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => TinyNeuralModel::default().parameter_vector(),
            };
            let population_size =
                parse_usize_arg(args.next(), NesConfig::default().population_size);
            let sigma = parse_f32_arg(args.next(), NesConfig::default().sigma);
            let learning_rate = parse_f32_arg(args.next(), NesConfig::default().learning_rate);
            let seed = parse_u64_arg(args.next(), NesConfig::default().seed);
            let simulations = parse_usize_arg(args.next(), 32);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let nes_config = NesConfig {
                population_size,
                sigma,
                learning_rate,
                seed,
            };
            let result = match run_nes_step(
                &base_vector,
                nes_config,
                build_mcts_config(simulations, cpuct),
            ) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let line = result
                .vector
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            if let Err(error) = fs::write(&output_vector_file, format!("{line}\n")) {
                eprintln!("failed to write output vector: {error}");
                std::process::exit(2);
            }
            println!("initial_fitness {}", result.initial_fitness);
            println!("best_candidate_fitness {}", result.best_candidate_fitness);
            println!("updated_fitness {}", result.updated_fitness);
            println!("evaluations {}", result.evaluations);
            println!("output_vector {}", output_vector_file);
        }
        "nes-train" => {
            let Some(output_vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let mut vector = match args.next() {
                Some(input_vector_file) => match load_parameter_vector(&input_vector_file) {
                    Ok(vector) => vector,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => TinyNeuralModel::default().parameter_vector(),
            };
            let generations = parse_usize_arg(args.next(), 8);
            let population_size =
                parse_usize_arg(args.next(), NesConfig::default().population_size);
            let sigma = parse_f32_arg(args.next(), NesConfig::default().sigma);
            let learning_rate = parse_f32_arg(args.next(), NesConfig::default().learning_rate);
            let base_seed = parse_u64_arg(args.next(), NesConfig::default().seed);
            let simulations = parse_usize_arg(args.next(), 32);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let mcts_config = build_mcts_config(simulations, cpuct);
            let mut total_evaluations = 0usize;

            for generation in 0..generations.max(1) {
                let result = match run_nes_step(
                    &vector,
                    NesConfig {
                        population_size,
                        sigma,
                        learning_rate,
                        seed: base_seed.wrapping_add(generation as u64),
                    },
                    mcts_config,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                };
                total_evaluations += result.evaluations;
                println!(
                    "generation {} initial_fitness={} best_candidate_fitness={} updated_fitness={}",
                    generation + 1,
                    result.initial_fitness,
                    result.best_candidate_fitness,
                    result.updated_fitness
                );
                vector = result.vector;
            }

            let final_evaluation = match evaluate_nes_fitness(&vector, mcts_config) {
                Ok(evaluation) => evaluation,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let line = vector
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            if let Err(error) = fs::write(&output_vector_file, format!("{line}\n")) {
                eprintln!("failed to write output vector: {error}");
                std::process::exit(2);
            }
            println!("final_fitness {}", final_evaluation.total_fitness);
            println!("total_evaluations {}", total_evaluations);
            println!("output_vector {}", output_vector_file);
        }
        "status" => {
            let fen = args
                .next()
                .unwrap_or_else(|| Position::STARTPOS_FEN.to_string());
            let position = match Position::from_fen(&fen) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("invalid fen: {error}");
                    std::process::exit(2);
                }
            };
            println!("{}", format_status(position.game_status()));
        }
        "apply" => {
            let Some(fen) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let Some(uci) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let position = match Position::from_fen(&fen) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("invalid fen: {error}");
                    std::process::exit(2);
                }
            };
            match position.apply_uci_move(&uci) {
                Some(next) => println!("{}", next.to_fen()),
                None => {
                    eprintln!("illegal or unsupported move");
                    std::process::exit(3);
                }
            }
        }
        "match-uci" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let garuda_color = match args.next() {
                Some(text) => match parse_color(&text) {
                    Ok(color) => color,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => garuda::chess::Color::White,
            };
            let garuda_depth = parse_usize_arg(args.next(), SearchConfig::default().max_depth);
            let garuda_quiescence =
                parse_usize_arg(args.next(), SearchConfig::default().quiescence_depth);

            let engine = Engine::new(
                TinyNeuralModel::default(),
                build_search_config(garuda_depth, garuda_quiescence),
            );
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let start_position = Position::starting_position();
            let (position, status, san_moves) = match play_match_game(
                |position, repetition_history| {
                    engine.best_move_with_history(position, repetition_history)
                },
                &mut uci,
                max_plies,
                movetime_ms,
                garuda_color,
                true,
                &start_position,
            ) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(3);
                }
            };
            print_match_pgn(
                "Garuda vs UCI",
                &position,
                status,
                &san_moves,
                garuda_color,
                "Garuda",
            );
        }
        "bo-uci" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let games = parse_usize_arg(args.next(), 10);
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let garuda_depth = parse_usize_arg(args.next(), SearchConfig::default().max_depth);
            let garuda_quiescence =
                parse_usize_arg(args.next(), SearchConfig::default().quiescence_depth);
            let openings = match args.next() {
                Some(path) => match load_openings(&path) {
                    Ok(openings) => openings,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => vec![Position::starting_position()],
            };

            let engine = Engine::new(
                TinyNeuralModel::default(),
                build_search_config(garuda_depth, garuda_quiescence),
            );
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };

            let mut garuda_wins = 0usize;
            let mut uci_wins = 0usize;
            let mut draws = 0usize;
            for game_index in 0..games {
                let garuda_color = if game_index % 2 == 0 {
                    garuda::chess::Color::White
                } else {
                    garuda::chess::Color::Black
                };
                let start_position = &openings[game_index % openings.len()];
                let (_position, status, _san_moves) = match play_match_game(
                    |position, repetition_history| {
                        engine.best_move_with_history(position, repetition_history)
                    },
                    &mut uci,
                    max_plies,
                    movetime_ms,
                    garuda_color,
                    false,
                    start_position,
                ) {
                    Ok(position) => position,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(3);
                    }
                };
                let result = summarize_bo_result(
                    status,
                    garuda_color,
                    &mut garuda_wins,
                    &mut uci_wins,
                    &mut draws,
                );
                println!(
                    "game {} color {} result {} status {}",
                    game_index + 1,
                    garuda_color.fen_symbol(),
                    result,
                    format_status(status)
                );
            }
            println!(
                "summary games={} garuda_wins={} uci_wins={} draws={}",
                games, garuda_wins, uci_wins, draws
            );
        }
        "match-uci-mcts" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let garuda_color = match args.next() {
                Some(text) => match parse_color(&text) {
                    Ok(color) => color,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => garuda::chess::Color::White,
            };
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let engine = MctsEngine::new(
                TinyNeuralModel::default(),
                build_mcts_config(simulations, cpuct),
            );
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let start_position = Position::starting_position();
            let (position, status, san_moves) = match play_match_game(
                |position, repetition_history| {
                    engine.best_move_with_history(position, repetition_history)
                },
                &mut uci,
                max_plies,
                movetime_ms,
                garuda_color,
                true,
                &start_position,
            ) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(3);
                }
            };
            print_match_pgn(
                "Garuda MCTS vs UCI",
                &position,
                status,
                &san_moves,
                garuda_color,
                "Garuda MCTS",
            );
        }
        "match-uci-mcts-vector" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let Some(vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let model = match load_model_from_vector_file(&vector_file) {
                Ok(model) => model,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let garuda_color = match args.next() {
                Some(text) => match parse_color(&text) {
                    Ok(color) => color,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => garuda::chess::Color::White,
            };
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let engine = MctsEngine::new(model, build_mcts_config(simulations, cpuct));
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let start_position = Position::starting_position();
            let (position, status, san_moves) = match play_match_game(
                |position, repetition_history| {
                    engine.best_move_with_history(position, repetition_history)
                },
                &mut uci,
                max_plies,
                movetime_ms,
                garuda_color,
                true,
                &start_position,
            ) {
                Ok(position) => position,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(3);
                }
            };
            print_match_pgn(
                "Garuda MCTS Vector vs UCI",
                &position,
                status,
                &san_moves,
                garuda_color,
                "Garuda MCTS",
            );
        }
        "bo-uci-mcts" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let games = parse_usize_arg(args.next(), 10);
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let openings = match args.next() {
                Some(path) => match load_openings(&path) {
                    Ok(openings) => openings,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => vec![Position::starting_position()],
            };
            let engine = MctsEngine::new(
                TinyNeuralModel::default(),
                build_mcts_config(simulations, cpuct),
            );
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let mut garuda_wins = 0usize;
            let mut uci_wins = 0usize;
            let mut draws = 0usize;
            for game_index in 0..games {
                let garuda_color = if game_index % 2 == 0 {
                    garuda::chess::Color::White
                } else {
                    garuda::chess::Color::Black
                };
                let start_position = &openings[game_index % openings.len()];
                let (_position, status, _san_moves) = match play_match_game(
                    |position, repetition_history| {
                        engine.best_move_with_history(position, repetition_history)
                    },
                    &mut uci,
                    max_plies,
                    movetime_ms,
                    garuda_color,
                    false,
                    start_position,
                ) {
                    Ok(position) => position,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(3);
                    }
                };
                let result = summarize_bo_result(
                    status,
                    garuda_color,
                    &mut garuda_wins,
                    &mut uci_wins,
                    &mut draws,
                );
                println!(
                    "game {} color {} result {} status {}",
                    game_index + 1,
                    garuda_color.fen_symbol(),
                    result,
                    format_status(status)
                );
            }
            println!(
                "summary games={} garuda_wins={} uci_wins={} draws={}",
                games, garuda_wins, uci_wins, draws
            );
        }
        "bo-uci-mcts-vector" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let Some(vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let model = match load_model_from_vector_file(&vector_file) {
                Ok(model) => model,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let games = parse_usize_arg(args.next(), 10);
            let max_plies = parse_optional_plies_arg(args.next(), None);
            let movetime_ms = parse_u64_arg(args.next(), 50);
            let simulations = parse_usize_arg(args.next(), MctsConfig::default().simulations);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let openings = match args.next() {
                Some(path) => match load_openings(&path) {
                    Ok(openings) => openings,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => vec![Position::starting_position()],
            };
            let engine = MctsEngine::new(model, build_mcts_config(simulations, cpuct));
            let mut uci = match UciEngine::spawn(&engine_command) {
                Ok(uci) => uci,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            };
            let mut garuda_wins = 0usize;
            let mut uci_wins = 0usize;
            let mut draws = 0usize;
            for game_index in 0..games {
                let garuda_color = if game_index % 2 == 0 {
                    garuda::chess::Color::White
                } else {
                    garuda::chess::Color::Black
                };
                let start_position = &openings[game_index % openings.len()];
                let (_position, status, _san_moves) = match play_match_game(
                    |position, repetition_history| {
                        engine.best_move_with_history(position, repetition_history)
                    },
                    &mut uci,
                    max_plies,
                    movetime_ms,
                    garuda_color,
                    false,
                    start_position,
                ) {
                    Ok(position) => position,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(3);
                    }
                };
                let result = summarize_bo_result(
                    status,
                    garuda_color,
                    &mut garuda_wins,
                    &mut uci_wins,
                    &mut draws,
                );
                println!(
                    "game {} color {} result {} status {}",
                    game_index + 1,
                    garuda_color.fen_symbol(),
                    result,
                    format_status(status)
                );
            }
            println!(
                "summary games={} garuda_wins={} uci_wins={} draws={}",
                games, garuda_wins, uci_wins, draws
            );
        }
        "nes-eval-uci" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let model = match args.next() {
                Some(vector_file) => match load_model_from_vector_file(&vector_file) {
                    Ok(model) => model,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => TinyNeuralModel::default(),
            };
            let games = parse_usize_arg(args.next(), 2);
            let max_plies = parse_optional_plies_arg(args.next(), Some(12));
            let movetime_ms = parse_u64_arg(args.next(), 10);
            let simulations = parse_usize_arg(args.next(), 16);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let openings = match args.next() {
                Some(path) => match load_openings(&path) {
                    Ok(openings) => openings,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => vec![Position::starting_position()],
            };
            let result = match evaluate_mcts_model_against_uci(
                model,
                &engine_command,
                &UciFitnessConfig {
                    games,
                    max_plies,
                    movetime_ms,
                    simulations,
                    cpuct,
                    openings,
                },
            ) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(3);
                }
            };
            println!("fitness {}", result.fitness);
            println!("garuda_wins {}", result.garuda_wins);
            println!("uci_wins {}", result.uci_wins);
            println!("draws {}", result.draws);
            for game in result.games {
                println!(
                    "game {} color {} result {} status {} plies {}",
                    game.game_index,
                    game.garuda_color,
                    game.result,
                    game.status,
                    game.plies
                );
            }
        }
        "nes-train-uci" => {
            let Some(engine_command) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let Some(output_vector_file) = args.next() else {
                print_usage();
                std::process::exit(1);
            };
            let input_vector_file = args.next();
            let mut vector = match input_vector_file.as_ref() {
                Some(input_vector_file) => match load_parameter_vector(input_vector_file) {
                    Ok(vector) => vector,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => TinyNeuralModel::default().parameter_vector(),
            };
            let generations = parse_usize_arg(args.next(), 4);
            let population_size = parse_usize_arg(args.next(), 4);
            let sigma = parse_f32_arg(args.next(), 0.03);
            let learning_rate = parse_f32_arg(args.next(), 0.02);
            let seed = parse_u64_arg(args.next(), 7);
            let games = parse_usize_arg(args.next(), 2);
            let max_plies = parse_optional_plies_arg(args.next(), Some(12));
            let movetime_ms = parse_u64_arg(args.next(), 10);
            let simulations = parse_usize_arg(args.next(), 16);
            let cpuct = parse_f32_arg(args.next(), MctsConfig::default().cpuct);
            let openings = match args.next() {
                Some(path) => match load_openings(&path) {
                    Ok(openings) => openings,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                },
                None => vec![Position::starting_position()],
            };
            let run_dir = args.next();
            let uci_config = UciFitnessConfig {
                games,
                max_plies,
                movetime_ms,
                simulations,
                cpuct,
                openings,
            };
            if let Some(run_dir) = run_dir.as_deref() {
                if let Err(error) = write_nes_train_run_config(
                    run_dir,
                    &engine_command,
                    &output_vector_file,
                    input_vector_file.as_deref(),
                    generations,
                    population_size,
                    sigma,
                    learning_rate,
                    seed,
                    games,
                    max_plies,
                    movetime_ms,
                    simulations,
                    cpuct,
                    uci_config.openings.len(),
                ) {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
            let mut total_evaluations = 0usize;
            let mut best_updated_fitness = f32::NEG_INFINITY;
            let mut best_generation = 0usize;
            let mut best_vector = vector.clone();
            for generation in 0..generations.max(1) {
                let (
                    initial_fitness,
                    best_candidate_fitness,
                    updated_fitness,
                    evaluations,
                    next_vector,
                ) = match run_nes_step_uci(
                    &vector,
                    NesConfig {
                        population_size,
                        sigma,
                        learning_rate,
                        seed: seed.wrapping_add(generation as u64),
                    },
                    &engine_command,
                    &uci_config,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(3);
                    }
                };
                total_evaluations += evaluations;
                println!(
                    "generation {} initial_fitness={} best_candidate_fitness={} updated_fitness={}",
                    generation + 1,
                    initial_fitness,
                    best_candidate_fitness,
                    updated_fitness
                );
                if let Some(run_dir) = run_dir.as_deref() {
                    let checkpoint_path =
                        format!("{run_dir}/generation-{:04}.vec", generation + 1);
                    let progress_path = format!("{run_dir}/progress.tsv");
                    if let Err(error) = write_vector_file(&checkpoint_path, &next_vector) {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                    if let Err(error) = append_file_line(
                        &progress_path,
                        &format!(
                            "{}\t{}\t{}\t{}\t{}\t{}",
                            generation + 1,
                            initial_fitness,
                            best_candidate_fitness,
                            updated_fitness,
                            total_evaluations,
                            checkpoint_path
                        ),
                    ) {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                }
                if updated_fitness > best_updated_fitness {
                    best_updated_fitness = updated_fitness;
                    best_generation = generation + 1;
                    best_vector = next_vector.clone();
                    if let Some(run_dir) = run_dir.as_deref() {
                        let best_path = format!("{run_dir}/best.vec");
                        if let Err(error) = write_vector_file(&best_path, &best_vector) {
                            eprintln!("{error}");
                            std::process::exit(2);
                        }
                    }
                }
                vector = next_vector;
            }
            if let Err(error) = write_vector_file(&output_vector_file, &best_vector) {
                eprintln!("{error}");
                std::process::exit(2);
            }
            if let Some(run_dir) = run_dir.as_deref() {
                let latest_path = format!("{run_dir}/latest.vec");
                if let Err(error) = write_vector_file(&latest_path, &vector) {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
            let final_result = match evaluate_mcts_model_against_uci(
                TinyNeuralModel::from_parameter_vector(&best_vector).unwrap_or_default(),
                &engine_command,
                &uci_config,
            ) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(3);
                }
            };
            if let Some(run_dir) = run_dir.as_deref() {
                let summary_path = format!("{run_dir}/summary.txt");
                let final_games_path = format!("{run_dir}/final-games.tsv");
                if let Err(error) = write_uci_game_records(&final_games_path, &final_result.games) {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
                let summary = format!(
                    "best_generation={}\nbest_updated_fitness={}\nfinal_fitness={}\ntotal_evaluations={}\ngaruda_wins={}\nuci_wins={}\ndraws={}\noutput_vector={}\nlatest_vector={run_dir}/latest.vec\nbest_vector={run_dir}/best.vec\nfinal_games={run_dir}/final-games.tsv\n",
                    best_generation,
                    best_updated_fitness,
                    final_result.fitness,
                    total_evaluations,
                    final_result.garuda_wins,
                    final_result.uci_wins,
                    final_result.draws,
                    output_vector_file
                );
                if let Err(error) = fs::write(&summary_path, summary) {
                    eprintln!("failed to write run summary {summary_path}: {error}");
                    std::process::exit(2);
                }
            }
            println!("final_fitness {}", final_result.fitness);
            println!("total_evaluations {}", total_evaluations);
            println!("garuda_wins {}", final_result.garuda_wins);
            println!("uci_wins {}", final_result.uci_wins);
            println!("draws {}", final_result.draws);
            println!("best_generation {}", best_generation);
            println!("best_updated_fitness {}", best_updated_fitness);
            println!("output_vector {}", output_vector_file);
            if let Some(run_dir) = run_dir.as_deref() {
                println!("run_dir {}", run_dir);
                println!("progress_file {run_dir}/progress.tsv");
                println!("summary_file {run_dir}/summary.txt");
                println!("final_games_file {run_dir}/final-games.tsv");
            }
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}
