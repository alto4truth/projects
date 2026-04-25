#include "garuda/Group/GroupDomainBuilder.h"
#include "garuda/Group/GroupDomain.h"
#include <stdexcept>

namespace garuda {

GroupDomainBuilder::GroupDomainBuilder() {
    config = GroupDomainConfig();
}

GroupDomainBuilder::~GroupDomainBuilder() = default;

GroupDomainBuilder& GroupDomainBuilder::setName(const std::string& n) {
    name = n;
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::setConfig(const GroupDomainConfig& cfg) {
    config = cfg;
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::addMember(const std::string& key, std::unique_ptr<AbstractDomain> domain) {
    members[key] = std::move(domain);
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::addDependency(const std::string& from, const std::string& to) {
    dependencies.push_back({from, to});
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::addObserver(std::shared_ptr<GroupDomainObserver> observer) {
    observers.push_back(observer);
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::enableWidening(bool enable) {
    config.enable_widening = enable;
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::enableNarrowing(bool enable) {
    config.enable_narrowing = enable;
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::enableProgressTracking(bool enable) {
    config.enable_progress_tracking = enable;
    return *this;
}

GroupDomainBuilder& GroupDomainBuilder::setMaxIterations(size_t max) {
    config.max_iterations = max;
    return *this;
}

std::unique_ptr<GroupDomain> GroupDomainBuilder::build() {
    if (name.empty()) {
        throw std::runtime_error("GroupDomainBuilder: name not set");
    }
    auto group = std::make_unique<GroupDomain>(name, config);
    for (auto& pair : members) {
        group->addMember(pair.first, pair.second->clone());
    }
    for (const auto& dep : dependencies) {
        group->addDependency(dep.first, dep.second);
    }
    for (const auto& obs : observers) {
        group->addObserver(obs);
    }
    return group;
}

std::shared_ptr<GroupDomain> GroupDomainBuilder::buildShared() {
    if (name.empty()) {
        throw std::runtime_error("GroupDomainBuilder: name not set");
    }
    auto group = std::make_shared<GroupDomain>(name, config);
    for (auto& pair : members) {
        group->addMember(pair.first, pair.second->clone());
    }
    for (const auto& dep : dependencies) {
        group->addDependency(dep.first, dep.second);
    }
    for (const auto& obs : observers) {
        group->addObserver(obs);
    }
    return group;
}

GroupDomainBuilder GroupDomainBuilder::create(const std::string& n) {
    GroupDomainBuilder builder;
    builder.setName(n);
    return builder;
}

}