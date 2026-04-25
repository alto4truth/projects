#ifndef GARUDA_GROUP_DOMAIN_MANAGER_H
#define GARUDA_GROUP_DOMAIN_MANAGER_H

#include "garuda/Group/GroupDomain.h"
#include <string>
#include <map>
#include <set>
#include <memory>
#include <functional>

namespace garuda {

class GroupDomainManager {
public:
    GroupDomainManager();
    ~GroupDomainManager();

    std::shared_ptr<GroupDomain> createGroup(const std::string& name);
    std::shared_ptr<GroupDomain> createGroup(const std::string& name, const GroupDomainConfig& config);
    std::shared_ptr<GroupDomain> getGroup(const std::string& name) const;
    bool hasGroup(const std::string& name) const;
    void removeGroup(const std::string& name);
    std::vector<std::string> getGroupNames() const;
    size_t getGroupCount() const;

    void registerGlobalObserver(std::shared_ptr<GroupDomainObserver> observer);
    void unregisterGlobalObserver(std::shared_ptr<GroupDomainObserver> observer);

    bool mergeGroups(const std::string& from, const std::string& to);
    bool splitGroup(const std::string& name, const std::vector<std::string>& subgroups);

    std::vector<std::string> topologicalSort() const;
    bool detectCycles() const;

    void reset();
    void resetGroup(const std::string& name);

    using GroupIterator = std::map<std::string, std::shared_ptr<GroupDomain>>::iterator;
    GroupIterator begin() { return groups.begin(); }
    GroupIterator end() { return groups.end(); }

private:
    std::map<std::string, std::shared_ptr<GroupDomain>> groups;
    std::vector<std::shared_ptr<GroupDomainObserver>> global_observers;
};

}
#endif