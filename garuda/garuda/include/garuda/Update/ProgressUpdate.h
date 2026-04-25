#ifndef GARUDA_PROGRESS_UPDATE_H
#define GARUDA_PROGRESS_UPDATE_H

#include <string>
#include <vector>
#include <map>
#include <set>
#include <memory>
#include <functional>
#include <chrono>
#include <optional>

namespace garuda {

enum class ProgressLevel {
    NONE,
    MINIMAL,
    NORMAL,
    DETAILED,
    VERBOSE
};

enum class UpdateStatus {
    PENDING,
    IN_PROGRESS,
    COMPLETED,
    FAILED,
    CANCELLED,
    STABLE
};

struct ProgressSnapshot {
    uint64_t iteration;
    uint64_t timestamp;
    double progress_ratio;
    std::map<std::string, std::string> state;
    std::vector<std::string> changed_keys;
    bool is_converging;
    double convergence_rate;

    ProgressSnapshot()
        : iteration(0),
          timestamp(0),
          progress_ratio(0.0),
          is_converging(false),
          convergence_rate(0.0) {}
};

struct ProgressDelta {
    std::string key;
    std::string old_value;
    std::string new_value;
    bool improved;
    double delta_magnitude;

    ProgressDelta() : improved(false), delta_magnitude(0.0) {}
};

class ProgressUpdate {
public:
    ProgressUpdate();
    explicit ProgressUpdate(const std::string& name);
    ~ProgressUpdate();

    void setName(const std::string& name);
    const std::string& getName() const { return name; }

    void setLevel(ProgressLevel level);
    ProgressLevel getLevel() const { return level; }

    void setStatus(UpdateStatus status);
    UpdateStatus getStatus() const { return status; }

    void recordIteration(uint64_t iter);
    uint64_t getIteration() const { return iteration; }

    void recordProgress(double progress);
    double getProgress() const { return progress; }

    void addDelta(const ProgressDelta& delta);
    const std::vector<ProgressDelta>& getDeltas() const { return deltas; }

    void addSnapshot(const ProgressSnapshot& snapshot);
    const std::vector<ProgressSnapshot>& getSnapshots() const { return snapshots; }

    bool hasImproved() const;
    bool isConverging() const;
    double getConvergenceRate() const;

    void markStable();
    bool isStable() const { return status == UpdateStatus::STABLE; }

    void setStabilityThreshold(double threshold);
    double getStabilityThreshold() const { return stability_threshold; }

    void reset();
    void clearHistory();

    ProgressSnapshot getCurrentSnapshot() const;
    std::vector<ProgressSnapshot> getRecentSnapshots(size_t count) const;

private:
    std::string name;
    ProgressLevel level;
    UpdateStatus status;
    uint64_t iteration;
    double progress;
    std::vector<ProgressDelta> deltas;
    std::vector<ProgressSnapshot> snapshots;
    double stability_threshold;
    std::chrono::steady_clock::time_point start_time;
};

}
#endif