#include "garuda/Group/GroupDomainManager.h"
#include "garuda/Group/GroupDomain.h"
#include <algorithm>
#include <stdexcept>

namespace garuda {

GroupDomainManager::GroupDomainManager() = default;

GroupDomainManager::~GroupDomainManager() = default;

std::shared_ptr<GroupDomain> GroupDomainManager::createGroup(const std::string& name) {
    return createGroup(name, GroupDomainConfig());
}

std::shared_ptr<GroupDomain> GroupDomainManager::createGroup(const std::string& name, const GroupDomainConfig& config) {
    auto group = std::make_shared<GroupDomain>(name, config);
    groups[name] = group;
    GroupEvent event(GroupEventType::DOMAIN_CREATED, name);
    for (const auto& obs : global_observers) {
        obs->onGroupEvent(event);
    }
    return group;
}

std::shared_ptr<GroupDomain> GroupDomainManager::getGroup(const std::string& name) const {
    auto it = groups.find(name);
    if (it != groups.end()) return it->second;
    return nullptr;
    return nullptr;
}

bool GroupDomainManager::hasGroup(const std::string& name) const {
    return groups.find(name) != groups.end();
}

void GroupDomainManager::removeGroup(const std::string& name) {
    groups.erase(name);
}

std::vector<std::string> GroupDomainManager::getGroupNames() const {
    std::vector<std::string> names;
    for (const auto& pair : groups) {
        names.push_back(pair.first);
    }
    return names;
}

size_t GroupDomainManager::getGroupCount() const {
    return groups.size();
}

void GroupDomainManager::registerGlobalObserver(std::shared_ptr<GroupDomainObserver> observer) {
    global_observers.push_back(observer);
}

void GroupDomainManager::unregisterGlobalObserver(std::shared_ptr<GroupDomainObserver> observer) {
    for (auto it = global_observers.begin(); it != global_observers.end(); ++it) {
        if (*it == observer) {
            global_observers.erase(it);
            break;
        }
    }
}

bool GroupDomainManager::mergeGroups(const std::string& from, const std::string& to) {
    auto from_it = groups.find(from);
    auto to_it = groups.find(to);
    if (from_it == groups.end() || to_it == groups.end()) {
        return false;
    }
    auto merged = from_it->second->join(*to_it->second);
    if (auto* merged_group = dynamic_cast<GroupDomain*>(merged.get())) {
        to_it->second = std::shared_ptr<GroupDomain>(merged_group);
        groups.erase(from_it);
        return true;
    }
    return false;
}

bool GroupDomainManager::splitGroup(const std::string& name, const std::vector<std::string>& subgroups) {
    auto it = groups.find(name);
    if (it == groups.end()) {
        return false;
    }
    auto group = it->second;
    for (const auto& subgroup_name : subgroups) {
        if (groups.find(subgroup_name) == groups.end()) {
            auto subgroup = std::make_shared<GroupDomain>(subgroup_name, group->getConfig());
            groups[subgroup_name] = subgroup;
        }
    }
    groups.erase(it);
    return true;
}

std::vector<std::string> GroupDomainManager::topologicalSort() const {
    std::vector<std::string> result;
    std::set<std::string> visited;
    std::set<std::string> temp;

    std::function<bool(const std::string&)> visit = [&](const std::string& name) -> bool {
        if (temp.find(name) != temp.end()) return false;
        if (visited.find(name) != visited.end()) return true;

        temp.insert(name);
        visited.insert(name);

        auto it = groups.find(name);
        if (it != groups.end()) {
            auto group = it->second;
            auto members = group->getMemberKeys();
            for (const auto& member : members) {
                if (groups.find(member) != groups.end() && !visit(member)) {
                    return false;
                }
            }
        }

        temp.erase(name);
        result.push_back(name);
        return true;
    };

    for (const auto& pair : groups) {
        if (visited.find(pair.first) == visited.end()) {
            if (!visit(pair.first)) {
                return {};
            }
        }
    }

    std::reverse(result.begin(), result.end());
    return result;
}

bool GroupDomainManager::detectCycles() const {
    std::set<std::string> visited;
    std::set<std::string> temp;

    std::function<bool(const std::string&)> visit = [&](const std::string& name) -> bool {
        if (temp.find(name) != temp.end()) return true;
        if (visited.find(name) != visited.end()) return false;

        temp.insert(name);
        visited.insert(name);

        auto it = groups.find(name);
        if (it != groups.end()) {
            auto group = it->second;
            auto members = group->getMemberKeys();
            for (const auto& member : members) {
                if (groups.find(member) != groups.end() && visit(member)) {
                    return true;
                }
            }
        }

        temp.erase(name);
        return false;
    };

    for (const auto& pair : groups) {
        if (visit(pair.first)) {
            return true;
        }
    }

    return false;
}

void GroupDomainManager::reset() {
    groups.clear();
    global_observers.clear();
}

void GroupDomainManager::resetGroup(const std::string& name) {
    auto it = groups.find(name);
    if (it != groups.end()) {
        it->second = std::make_shared<GroupDomain>(name);
    }
}

}