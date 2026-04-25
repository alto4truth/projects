#include "garuda/Update/ProgressUpdateTracker.h"
#include "garuda/Update/ProgressUpdate.h"
#include <algorithm>
#include <cmath>

namespace garuda {

ProgressUpdateTracker::ProgressUpdateTracker()
    : current_iteration(0),
      total_iterations(0),
      convergence_threshold(0.001),
      max_stagnation(3),
      track_history(true),
      enable_timestamps(true) {
    start_time = std::chrono::steady_clock::now();
}

ProgressUpdateTracker::ProgressUpdateTracker(const std::string& name)
    : name(name),
      current_iteration(0),
      total_iterations(0),
      convergence_threshold(0.001),
      max_stagnation(3),
      track_history(true),
      enable_timestamps(true) {
    start_time = std::chrono::steady_clock::now();
}

ProgressUpdateTracker::~ProgressUpdateTracker() = default;

void ProgressUpdateTracker::initialize(size_t total) {
    total_iterations = total;
    current_iteration = 0;
    history.clear();
    checkpoints.clear();
    stagnation_count = 0;
    last_progress = 0.0;
    start_time = std::chrono::steady_clock::now();
}

void ProgressUpdateTracker::recordIteration(size_t iter, double progress) {
    current_iteration = iter;
    ProgressUpdate update;
    update.setName(name);
    update.recordIteration(iter);
    update.recordProgress(progress);

    auto now = std::chrono::steady_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(now - start_time);
    uint64_t timestamp = duration.count();

    Entry entry;
    entry.iteration = iter;
    entry.progress = progress;
    entry.timestamp = timestamp;
    entry.delta = std::fabs(progress - last_progress);
    entry.is_stagnant = entry.delta < convergence_threshold;

    if (entry.is_stagnant) {
        ++stagnation_count;
    } else {
        stagnation_count = 0;
    }
    entry.is_converging = stagnation_count >= max_stagnation;

    last_progress = progress;

    if (track_history) {
        history.push_back(entry);
        if (history.size() > max_history_size) {
            history.erase(history.begin());
        }
    }

    createCheckpointIfNeeded();
}

bool ProgressUpdateTracker::hasConverged() const {
    if (history.empty()) return false;
    return history.back().is_converging;
}

bool ProgressUpdateTracker::shouldTerminate() const {
    if (current_iteration >= total_iterations && total_iterations > 0) return true;
    if (hasConverged()) return true;
    if (stagnation_count >= max_stagnation) return true;
    return false;
}

double ProgressUpdateTracker::getProgress() const {
    if (history.empty()) return 0.0;
    return history.back().progress;
}

size_t ProgressUpdateTracker::getCurrentIteration() const {
    return current_iteration;
}

size_t ProgressUpdateTracker::getTotalIterations() const {
    return total_iterations;
}

double ProgressUpdateTracker::computeProgressRate() const {
    if (history.size() < 2) return 0.0;
    double sum = 0.0;
    for (size_t i = 1; i < history.size(); ++i) {
        sum += history[i].delta;
    }
    return sum / (history.size() - 1);
}

double ProgressUpdateTracker::computeConvergenceRate() const {
    if (history.size() < 2) return 0.0;
    double sum = 0.0;
    size_t count = 0;
    for (size_t i = history.size() - std::min(history.size(), size_t(10)); i < history.size(); ++i) {
        sum += history[i].delta;
        ++count;
    }
    return count > 0 ? sum / count : 0.0;
}

void ProgressUpdateTracker::createCheckpoint() {
    Checkpoint cp;
    cp.iteration = current_iteration;
    cp.progress = last_progress;
    cp.history = history;
    cp.stagnation_count = stagnation_count;
    checkpoints.push_back(cp);
}

void ProgressUpdateTracker::createCheckpointIfNeeded() {
    if (checkpoints.empty() ||
        current_iteration - checkpoints.back().iteration >= checkpoint_interval) {
        createCheckpoint();
    }
}

void ProgressUpdateTracker::restoreCheckpoint(size_t index) {
    if (index < checkpoints.size()) {
        const auto& cp = checkpoints[index];
        current_iteration = cp.iteration;
        last_progress = cp.progress;
        history = cp.history;
        stagnation_count = cp.stagnation_count;
    }
}

void ProgressUpdateTracker::restoreLatestCheckpoint() {
    if (!checkpoints.empty()) {
        restoreCheckpoint(checkpoints.size() - 1);
    }
}

std::vector<ProgressUpdateTracker::Entry> ProgressUpdateTracker::getRecentHistory(size_t count) const {
    if (count >= history.size()) {
        return history;
    }
    std::vector<Entry> result;
    size_t start = history.size() - count;
    for (size_t i = start; i < history.size(); ++i) {
        result.push_back(history[i]);
    }
    return result;
}

void ProgressUpdateTracker::reset() {
    history.clear();
    checkpoints.clear();
    current_iteration = 0;
    stagnation_count = 0;
    last_progress = 0.0;
    start_time = std::chrono::steady_clock::now();
}

void ProgressUpdateTracker::setConvergenceThreshold(double threshold) {
    convergence_threshold = threshold;
}

void ProgressUpdateTracker::setMaxStagnation(size_t max) {
    max_stagnation = max;
}

void ProgressUpdateTracker::setCheckpointInterval(size_t interval) {
    checkpoint_interval = interval;
}

void ProgressUpdateTracker::enableHistory(bool enable) {
    track_history = enable;
}

size_t ProgressUpdateTracker::getHistorySize() const {
    return history.size();
}

size_t ProgressUpdateTracker::getCheckpointCount() const {
    return checkpoints.size();
}

double ProgressUpdateTracker::getElapsedTime() const {
    auto now = std::chrono::steady_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(now - start_time);
    return duration.count() / 1000.0;
}

double ProgressUpdateTracker::estimateTimeRemaining() const {
    if (history.size() < 2 || getProgress() <= 0.0) return 0.0;
    double elapsed = getElapsedTime();
    double progress = getProgress();
    return (elapsed / progress) * (1.0 - progress);
}

}