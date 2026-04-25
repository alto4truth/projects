#ifndef GARUDA_SUBSEQUENT_PROGRESS_UPDATE_H
#define GARUDA_SUBSEQUENT_PROGRESS_UPDATE_H

#include "garuda/Update/ProgressUpdate.h"
#include "garuda/Group/GroupDomain.h"
#include <memory>
#include <deque>
#include <unordered_map>

namespace garuda {

struct UpdateContext {
    std::string group_name;
    std::string analysis_name;
    uint64_t pass_id;
    uint64_t iteration;
    bool is_first_update;
    bool is_final_update;

    UpdateContext()
        : pass_id(0),
          iteration(0),
          is_first_update(false),
          is_final_update(false) {}
};

class SubsequentProgressUpdate : public ProgressUpdate {
public:
    SubsequentProgressUpdate();
    explicit SubsequentProgressUpdate(const std::string& name);
    ~SubsequentProgressUpdate();

    void initialize(const std::string& group_name, size_t expected_iterations);

    void recordUpdate(const UpdateContext& context,
                  const std::map<std::string, std::unique_ptr<AbstractDomain>>& old_state,
                  const std::map<std::string, std::unique_ptr<AbstractDomain>>& new_state);

    bool detectChange(const std::string& key) const;
    std::vector<std::string> getChangedKeys() const;

    bool hasConverged() const;
    bool shouldTerminate() const;

    void setMaxIterations(size_t max);
    size_t getMaxIterations() const { return max_iterations; }

    void setConvergenceThreshold(double threshold);
    double getConvergenceThreshold() const { return convergence_threshold; }

    double computeChangeMagnitude() const;
    double computeProgressRate() const;

    std::unique_ptr<SubsequentProgressUpdate> clone() const;
    std::unique_ptr<SubsequentProgressUpdate> createCheckpoint() const;
    void restoreFromCheckpoint(const SubsequentProgressUpdate& checkpoint);

    void setGroupDomain(std::shared_ptr<GroupDomain> domain);
    std::shared_ptr<GroupDomain> getGroupDomain() const { return group_domain; }

    void registerCustomMetric(const std::string& name,
                         std::function<double()> evaluator);
    double evaluateMetric(const std::string& name) const;
    std::vector<std::string> getMetricNames() const;

    struct HistoryEntry {
        uint64_t iteration;
        std::map<std::string, std::string> key_values;
        double change_magnitude;
        double progress_rate;
        bool is_converging;
    };

    const HistoryEntry* getHistory(size_t iteration) const;
    std::vector<HistoryEntry> getRecentHistory(size_t count) const;
    size_t getHistorySize() const { return history.size(); }

private:
    bool checkConvergence() const;
    void pruneHistory(size_t max_size);

    std::shared_ptr<GroupDomain> group_domain;
    size_t max_iterations;
    double convergence_threshold;
    std::deque<HistoryEntry> history;
    std::unordered_map<std::string, std::function<double()>> custom_metrics;
    std::map<std::string, std::string> previous_state;
    std::unique_ptr<SubsequentProgressUpdate> checkpoint;
};

}
#endif