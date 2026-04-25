#include "garuda/Update/ProgressUpdate.h"
#include <algorithm>
#include <cmath>

namespace garuda {

ProgressUpdate::ProgressUpdate()
    : name(""),
      level(ProgressLevel::NORMAL),
      status(UpdateStatus::PENDING),
      iteration(0),
      progress(0.0),
      stability_threshold(0.001) {
    start_time = std::chrono::steady_clock::now();
}

ProgressUpdate::ProgressUpdate(const std::string& n)
    : name(n),
      level(ProgressLevel::NORMAL),
      status(UpdateStatus::PENDING),
      iteration(0),
      progress(0.0),
      stability_threshold(0.001) {
    start_time = std::chrono::steady_clock::now();
}

ProgressUpdate::~ProgressUpdate() = default;

void ProgressUpdate::setName(const std::string& n) {
    name = n;
}

void ProgressUpdate::setLevel(ProgressLevel lvl) {
    level = lvl;
}

void ProgressUpdate::setStatus(UpdateStatus s) {
    status = s;
}

void ProgressUpdate::recordIteration(uint64_t iter) {
    iteration = iter;
}

void ProgressUpdate::recordProgress(double p) {
    progress = std::max(0.0, std::min(1.0, p));
}

void ProgressUpdate::addDelta(const ProgressDelta& delta) {
    deltas.push_back(delta);
}

void ProgressUpdate::addSnapshot(const ProgressSnapshot& snapshot) {
    snapshots.push_back(snapshot);
}

bool ProgressUpdate::hasImproved() const {
    for (const auto& delta : deltas) {
        if (delta.improved) return true;
    }
    return false;
}

bool ProgressUpdate::isConverging() const {
    if (snapshots.size() < 2) return false;
    const auto& recent = snapshots.back();
    return recent.is_converging;
}

double ProgressUpdate::getConvergenceRate() const {
    if (snapshots.size() < 2) return 0.0;
    double sum = 0.0;
    size_t count = 0;
    for (size_t i = 1; i < snapshots.size(); ++i) {
        double delta = std::fabs(snapshots[i].progress_ratio - snapshots[i-1].progress_ratio);
        sum += delta;
        ++count;
    }
    return count > 0 ? sum / count : 0.0;
}

void ProgressUpdate::markStable() {
    status = UpdateStatus::STABLE;
}

void ProgressUpdate::setStabilityThreshold(double threshold) {
    stability_threshold = threshold;
}

void ProgressUpdate::reset() {
    status = UpdateStatus::PENDING;
    iteration = 0;
    progress = 0.0;
    deltas.clear();
    start_time = std::chrono::steady_clock::now();
}

void ProgressUpdate::clearHistory() {
    snapshots.clear();
}

ProgressSnapshot ProgressUpdate::getCurrentSnapshot() const {
    ProgressSnapshot snapshot;
    snapshot.iteration = iteration;
    snapshot.progress_ratio = progress;
    auto now = std::chrono::steady_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(now - start_time);
    snapshot.timestamp = duration.count();
    for (const auto& delta : deltas) {
        snapshot.changed_keys.push_back(delta.key);
    }
    snapshot.is_converging = isConverging();
    snapshot.convergence_rate = getConvergenceRate();
    return snapshot;
}

std::vector<ProgressSnapshot> ProgressUpdate::getRecentSnapshots(size_t count) const {
    if (count >= snapshots.size()) return snapshots;
    std::vector<ProgressSnapshot> result;
    size_t start = snapshots.size() - count;
    for (size_t i = start; i < snapshots.size(); ++i) {
        result.push_back(snapshots[i]);
    }
    return result;
}

}