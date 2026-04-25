#ifndef GARUDA_PROGRESS_UPDATE_NOTIFIER_H
#define GARUDA_PROGRESS_UPDATE_NOTIFIER_H

#include "garuda/Update/ProgressUpdate.h"
#include <string>
#include <vector>
#include <memory>
#include <functional>
#include <atomic>

namespace garuda {

using ProgressCallback = std::function<void(const ProgressUpdate&)>;

class ProgressUpdateNotifier {
public:
    ProgressUpdateNotifier();
    ~ProgressUpdateNotifier();

    void subscribe(const std::string& name, ProgressCallback callback);
    void unsubscribe(const std::string& name);

    void notify(const ProgressUpdate& update);
    void notifyIteration(uint64_t iteration, double progress);
    void notifyChange(const std::string& key, const std::string& old_value, const std::string& new_value);
    void notifyConvergence(bool converged);
    void notifyFailure(const std::string& error);

    void setProgressLevel(ProgressLevel level);
    ProgressLevel getProgressLevel() const { return level; }

    void enable();
    void disable();
    bool isEnabled() const { return enabled; }

    void setMinimalInterval(double seconds);
    double getMinimalInterval() const { return minimal_interval; }

    void flush();

    size_t getSubscriberCount() const { return subscribers.size(); }
    std::vector<std::string> getSubscriberNames() const;

    class ProgressObserver {
    public:
        virtual ~ProgressObserver() = default;
        virtual void onProgressUpdate(const ProgressUpdate& update) = 0;
        virtual void onProgressChange(const std::string& key,
                                const std::string& old_value,
                                const std::string& new_value) = 0;
        virtual void onConvergence(bool converged) = 0;
        virtual void onFailure(const std::string& error) = 0;
    };

    void registerObserver(std::shared_ptr<ProgressObserver> observer);
    void unregisterObserver(std::shared_ptr<ProgressObserver> observer);

private:
    bool shouldNotify() const;
    double timeSinceLastNotification() const;

    ProgressLevel level;
    bool enabled;
    double minimal_interval;
    std::chrono::steady_clock::time_point last_notification;
    std::map<std::string, ProgressCallback> subscribers;
    std::vector<std::shared_ptr<ProgressObserver>> observers;
    std::atomic<uint64_t> notification_count;
};

}
#endif