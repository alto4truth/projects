#ifndef GARUDA_GROUP_DOMAIN_BUILDER_H
#define GARUDA_GROUP_DOMAIN_BUILDER_H

#include "garuda/Group/GroupDomain.h"
#include <string>
#include <vector>
#include <memory>
#include <functional>

namespace garuda {

class GroupDomainBuilder {
public:
    GroupDomainBuilder();
    ~GroupDomainBuilder();

    GroupDomainBuilder& setName(const std::string& name);
    GroupDomainBuilder& setConfig(const GroupDomainConfig& config);
    GroupDomainBuilder& addMember(const std::string& key, std::unique_ptr<AbstractDomain> domain);
    GroupDomainBuilder& addDependency(const std::string& from, const std::string& to);
    GroupDomainBuilder& addObserver(std::shared_ptr<GroupDomainObserver> observer);
    GroupDomainBuilder& enableWidening(bool enable = true);
    GroupDomainBuilder& enableNarrowing(bool enable = true);
    GroupDomainBuilder& enableProgressTracking(bool enable = true);
    GroupDomainBuilder& setMaxIterations(size_t max);

    std::unique_ptr<GroupDomain> build();
    std::shared_ptr<GroupDomain> buildShared();

    static GroupDomainBuilder create(const std::string& name);

private:
    std::string name;
    GroupDomainConfig config;
    std::map<std::string, std::unique_ptr<AbstractDomain>> members;
    std::vector<std::pair<std::string, std::string>> dependencies;
    std::vector<std::shared_ptr<GroupDomainObserver>> observers;
};

}
#endif