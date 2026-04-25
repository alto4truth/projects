#ifndef GARUDA_PROGRESS_UPDATE_TRACKER_H
#define GARUDA_PROGRESS_UPDATE_TRACKER_H

#include "garuda/Update/SubsequentProgressUpdate.h"
#include <string>
#include <map>
#include <memory>
#include <unordered_map>
#include <functional>

namespace garuda {

class ProgressUpdateTracker {
public:
    ProgressUpdateTracker();
    ~ProgressUpdateTracker();

    std::shared_ptr<SubsequentProgressUpdate> createTracker(const std::string& name);
    std::shared_ptr<SubsequentProgressUpdate> createTracker(const std::string& name,
                                               const std::string& group_name);
    std::shared_ptr<SubsequentProgressUpdate> getTracker(const std::string& name) const;
    bool hasTracker(const std::string& name) const;
    void removeTracker(const std::string& name);
    std::vector<std::string> getTrackerNames() const;
    size_t getTrackerCount() const { return trackers.size(); }

    void updateAll(const UpdateContext& context);
    void updateTracker(const std::string& name, const UpdateContext& context);

    void setGlobalConvergenceThreshold(double threshold);
    double getGlobalConvergenceThreshold() const { return global_convergence_threshold; }

    bool checkGlobalConvergence() const;
    bool hasConverged(const std::string& name) const;

    std::map<std::string, bool> getConvergenceStatus() const;

    void setOnUpdateCallback(const std::string& name,
                        std::function<void(const SubsequentProgressUpdate&)> callback);
    void setOnConvergenceCallback(const std::string& name,
                            std::function<void(const std::string&)> callback);

    void pauseTracking(const std::string& name);
    void resumeTracking(const std::string& name);
    bool isTracking(const std::string& name) const;

    void resetTracker(const std::string& name);
    void resetAll();

    using TrackerIterator = std::unordered_map<std::string,
                                     std::shared_ptr<SubsequentProgressUpdate>>::iterator;
    TrackerIterator begin() { return trackers.begin(); }
    TrackerIterator end() { return trackers.end(); }

private:
    std::unordered_map<std::string, std::shared_ptr<SubsequentProgressUpdate>> trackers;
    std::unordered_map<std::string, std::function<void(const SubsequentProgressUpdate&)>> update_callbacks;
    std::unordered_map<std::string, std::function<void(const std::string&)>> convergence_callbacks;
    std::unordered_map<std::string, bool> tracking_enabled;
    double global_convergence_threshold;
};

}
#endif