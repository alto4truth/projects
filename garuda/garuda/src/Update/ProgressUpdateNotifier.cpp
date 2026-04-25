#include "garuda/Update/ProgressUpdateNotifier.h"
#include "garuda/Update/ProgressUpdate.h"
#include <algorithm>
#include <chrono>

namespace garuda {

ProgressUpdateNotifier::ProgressUpdateNotifier()
    : level(ProgressLevel::NORMAL),
      enabled(true),
      minimal_interval(0.1),
      notification_count(0) {
    last_notification = std::chrono::steady_clock::now();
}

ProgressUpdateNotifier::~ProgressUpdateNotifier() = default;

void ProgressUpdateNotifier::subscribe(const std::string& name, ProgressCallback callback) {
    subscribers[name] = std::move(callback);
}

void ProgressUpdateNotifier::unsubscribe(const std::string& name) {
    subscribers.erase(name);
}

void ProgressUpdateNotifier::notify(const ProgressUpdate& update) {
    if (!shouldNotify()) return;
    ++notification_count;
    last_notification = std::chrono::steady_clock::now();
    for (auto& pair : subscribers) {
        if (pair.second) {
            pair.second(update);
        }
    }
    for (const auto& obs : observers) {
        obs->onProgressUpdate(update);
    }
}

void ProgressUpdateNotifier::notifyIteration(uint64_t iteration, double progress) {
    ProgressUpdate update;
    update.recordIteration(iteration);
    update.recordProgress(progress);
    notify(update);
}

void ProgressUpdateNotifier::notifyChange(const std::string& key, const std::string& old_value, const std::string& new_value) {
    for (const auto& obs : observers) {
        obs->onProgressChange(key, old_value, new_value);
    }
}

void ProgressUpdateNotifier::notifyConvergence(bool converged) {
    for (const auto& obs : observers) {
        obs->onConvergence(converged);
    }
}

void ProgressUpdateNotifier::notifyFailure(const std::string& error) {
    for (const auto& obs : observers) {
        obs->onFailure(error);
    }
}

void ProgressUpdateNotifier::setProgressLevel(ProgressLevel lvl) {
    level = lvl;
}

void ProgressUpdateNotifier::enable() {
    enabled = true;
}

void ProgressUpdateNotifier::disable() {
    enabled = false;
}

void ProgressUpdateNotifier::setMinimalInterval(double seconds) {
    minimal_interval = seconds;
}

double ProgressUpdateNotifier::getMinimalInterval() const {
    return minimal_interval;
}

void ProgressUpdateNotifier::flush() {
    for (auto& pair : subscribers) {
        if (pair.second) {
            ProgressUpdate update("flush");
            pair.second(update);
        }
    }
}

std::vector<std::string> ProgressUpdateNotifier::getSubscriberNames() const {
    std::vector<std::string> names;
    for (const auto& pair : subscribers) {
        names.push_back(pair.first);
    }
    return names;
}

void ProgressUpdateNotifier::registerObserver(std::shared_ptr<ProgressObserver> observer) {
    observers.push_back(observer);
}

void ProgressUpdateNotifier::unregisterObserver(std::shared_ptr<ProgressObserver> observer) {
    for (auto it = observers.begin(); it != observers.end(); ++it) {
        if (*it == observer) {
            observers.erase(it);
            break;
        }
    }
}

bool ProgressUpdateNotifier::shouldNotify() const {
    if (!enabled) return false;
    return timeSinceLastNotification() >= minimal_interval;
}

double ProgressUpdateNotifier::timeSinceLastNotification() const {
    auto now = std::chrono::steady_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::duration<double>>(now - last_notification);
    return duration.count();
}

}