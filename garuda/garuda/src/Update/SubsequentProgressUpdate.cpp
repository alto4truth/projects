#include "garuda/Update/SubsequentProgressUpdate.h"
#include "garuda/Update/ProgressUpdate.h"
#include "garuda/Group/GroupDomain.h"
#include <algorithm>
#include <cmath>

namespace garuda {

SubsequentProgressUpdate::SubsequentProgressUpdate()
    : ProgressUpdate(),
      group_domain(nullptr),
      max_iterations(100),
      convergence_threshold(0.001) {}

SubsequentProgressUpdate::SubsequentProgressUpdate(const std::string& name)
    : ProgressUpdate(name),
      group_domain(nullptr),
      max_iterations(100),
      convergence_threshold(0.001) {}

SubsequentProgressUpdate::~SubsequentProgressUpdate() = default;

void SubsequentProgressUpdate::initialize(const std::string& group_name, size_t expected_iterations) {
    setName(group_name);
    max_iterations = expected_iterations;
    recordIteration(0);
    recordProgress(0.0);
}

void SubsequentProgressUpdate::recordUpdate(
    const UpdateContext& context,
    const std::map<std::string, std::unique_ptr<AbstractDomain>>& old_state,
    const std::map<std::string, std::unique_ptr<AbstractDomain>>& new_state) {
    recordIteration(context.iteration);

    HistoryEntry entry;
    entry.iteration = context.iteration;
    entry.is_converging = false;

    if (!old_state.empty() && !new_state.empty()) {
        double change_sum = 0.0;
        for (const auto& pair : new_state) {
            auto it = old_state.find(pair.first);
            if (it != old_state.end()) {
                if (!pair.second->isEqual(*it->second)) {
                    entry.key_values[pair.first] = pair.second->toString();
                    change_sum += 1.0;
                }
            } else {
                entry.key_values[pair.first] = pair.second->toString();
                change_sum += 1.0;
            }
        }
        entry.change_magnitude = change_sum;
    }

    entry.progress_rate = computeProgressRate();
    if (history.size() >= 2) {
        entry.is_converging = entry.change_magnitude <= history.back().change_magnitude;
    }

    history.push_back(entry);
    if (history.size() > 1000) {
        pruneHistory(500);
    }

    recordProgress(computeProgressRate());

    if (hasConverged()) {
        markStable();
    }
}

bool SubsequentProgressUpdate::detectChange(const std::string& key) const {
    if (history.empty()) return false;
    const auto& entry = history.back();
    return entry.key_values.find(key) != entry.key_values.end();
}

std::vector<std::string> SubsequentProgressUpdate::getChangedKeys() const {
    std::vector<std::string> keys;
    if (!history.empty()) {
        for (const auto& pair : history.back().key_values) {
            keys.push_back(pair.first);
        }
    }
    return keys;
}

bool SubsequentProgressUpdate::hasConverged() const {
    return checkConvergence();
}

bool SubsequentProgressUpdate::shouldTerminate() const {
    if (getIteration() >= max_iterations) return true;
    if (hasConverged()) return true;
    if (isStable()) return true;
    return false;
}

void SubsequentProgressUpdate::setMaxIterations(size_t max) {
    max_iterations = max;
}

void SubsequentProgressUpdate::setConvergenceThreshold(double threshold) {
    convergence_threshold = threshold;
}

double SubsequentProgressUpdate::computeChangeMagnitude() const {
    if (history.empty()) return 0.0;
    return history.back().change_magnitude;
}

double SubsequentProgressUpdate::computeProgressRate() const {
    if (history.size() < 2) return 0.0;
    double sum = 0.0;
    for (size_t i = 1; i < history.size(); ++i) {
        sum += history[i].change_magnitude;
    }
    return sum / (history.size() - 1);
}

std::unique_ptr<SubsequentProgressUpdate> SubsequentProgressUpdate::clone() const {
    return std::make_unique<SubsequentProgressUpdate>(*this);
}

std::unique_ptr<SubsequentProgressUpdate> SubsequentProgressUpdate::createCheckpoint() const {
    auto checkpoint = std::make_unique<SubsequentProgressUpdate>();
    checkpoint->history = history;
    checkpoint->max_iterations = max_iterations;
    checkpoint->convergence_threshold = convergence_threshold;
    checkpoint->previous_state = previous_state;
    checkpoint->group_domain = group_domain;
    return checkpoint;
}

void SubsequentProgressUpdate::restoreFromCheckpoint(const SubsequentProgressUpdate& cp) {
    history = cp.history;
    max_iterations = cp.max_iterations;
    convergence_threshold = cp.convergence_threshold;
    previous_state = cp.previous_state;
    group_domain = cp.group_domain;
}

void SubsequentProgressUpdate::setGroupDomain(std::shared_ptr<GroupDomain> domain) {
    group_domain = domain;
}

void SubsequentProgressUpdate::registerCustomMetric(const std::string& name, std::function<double()> evaluator) {
    custom_metrics[name] = std::move(evaluator);
}

double SubsequentProgressUpdate::evaluateMetric(const std::string& name) const {
    auto it = custom_metrics.find(name);
    if (it != custom_metrics.end()) {
        return it->second();
    }
    return 0.0;
}

std::vector<std::string> SubsequentProgressUpdate::getMetricNames() const {
    std::vector<std::string> names;
    for (const auto& pair : custom_metrics) {
        names.push_back(pair.first);
    }
    return names;
}

const SubsequentProgressUpdate::HistoryEntry* SubsequentProgressUpdate::getHistory(size_t iteration) const {
    for (const auto& entry : history) {
        if (entry.iteration == iteration) {
            return &entry;
        }
    }
    return nullptr;
}

std::vector<SubsequentProgressUpdate::HistoryEntry> SubsequentProgressUpdate::getRecentHistory(size_t count) const {
    if (count >= history.size()) {
        return std::vector<HistoryEntry>(history.begin(), history.end());
    }
    std::vector<HistoryEntry> result;
    size_t start = history.size() - count;
    for (size_t i = start; i < history.size(); ++i) {
        result.push_back(history[i]);
    }
    return result;
}

bool SubsequentProgressUpdate::checkConvergence() const {
    if (history.size() < 3) return false;
    double recent_avg = 0.0;
    size_t count = 0;
    for (size_t i = history.size() - 3; i < history.size(); ++i) {
        recent_avg += history[i].change_magnitude;
        ++count;
    }
    recent_avg /= count;
    return recent_avg <= convergence_threshold;
}

void SubsequentProgressUpdate::pruneHistory(size_t max_size) {
    if (history.size() > max_size) {
        history.erase(history.begin(), history.end() - max_size);
    }
}

}