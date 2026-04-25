#ifndef GARUDA_GROUP_DOMAIN_H
#define GARUDA_GROUP_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <map>
#include <set>
#include <vector>
#include <memory>

namespace garuda {

struct GroupDomainConfig {
    bool enable_widening;
    bool enable_narrowing;
    size_t max_iterations;
    size_t widening_delay;
    bool track_dependencies;
    bool enable_invariant_checking;
    bool enable_progress_tracking;

    GroupDomainConfig()
        : enable_widening(true),
          enable_narrowing(true),
          max_iterations(100),
          widening_delay(3),
          track_dependencies(true),
          enable_invariant_checking(true),
          enable_progress_tracking(true) {}
};

enum class GroupEventType {
    DOMAIN_CREATED,
    DOMAIN_UPDATED,
    DOMAIN_MERGED,
    DOMAIN_SPLIT,
    DOMAIN_WIDENED,
    DOMAIN_NARROWED,
    DOMAIN_JOINED,
    DOMAIN_MEET,
    PROGRESS_UPDATE,
    DEPENDENCY_CHANGE,
    INVARIANT_VIOLATION,
    STABLE_STATE_REACHED,
    CONVERGENCE_DETECTED
};

struct GroupEvent {
    GroupEventType type;
    std::string group_name;
    std::string source_location;
    uint64_t timestamp;
    std::map<std::string, std::string> metadata;

    GroupEvent(GroupEventType ty, const std::string& name)
        : type(ty), group_name(name), timestamp(0) {}
};

class GroupDomainObserver {
public:
    virtual ~GroupDomainObserver() = default;
    virtual void onGroupEvent(const GroupEvent& event) = 0;
};

class GroupDomain : public AbstractDomain {
public:
    GroupDomain();
    explicit GroupDomain(const std::string& name);
    explicit GroupDomain(const std::string& name, const GroupDomainConfig& config);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::GROUP; }

    bool isTop() const override;
    bool isBottom() const override;
    bool isEqual(const AbstractDomain& other) const override;
    bool isLessThanOrEqual(const AbstractDomain& other) const override;

    std::unique_ptr<AbstractDomain> join(const AbstractDomain& other) const override;
    std::unique_ptr<AbstractDomain> meet(const AbstractDomain& other) const override;
    std::unique_ptr<AbstractDomain> widen(const AbstractDomain& other) const override;
    std::unique_ptr<AbstractDomain> narrow(const AbstractDomain& other) const override;

    std::unique_ptr<AbstractDomain> applyUnaryOp(DomainOperator op) const override;
    std::unique_ptr<AbstractDomain> applyBinaryOp(DomainOperator op, const AbstractDomain& other) const override;
    std::unique_ptr<AbstractDomain> applyCompare(DomainOperator cmp, const AbstractDomain& other) const override;

    void print(llvm::raw_ostream& os) const override;
    std::string toString() const override;

    DomainState getState() const override;
    void setState(const DomainState& state) override;

    llvm::APSInt getAbstractInteger() const override;
    llvm::APFloat getAbstractFloat() const override;
    bool getAbstractBoolean() const override;

    void setTop() override;
    void setBottom() override;

    size_t getHash() const override;

    void addMember(const std::string& key, std::unique_ptr<AbstractDomain> domain);
    std::unique_ptr<AbstractDomain> getMember(const std::string& key) const;
    bool hasMember(const std::string& key) const;
    void removeMember(const std::string& key);
    std::vector<std::string> getMemberKeys() const;
    size_t getMemberCount() const;

    void addDependency(const std::string& from, const std::string& to);
    std::vector<std::string> getDependencies(const std::string& key) const;
    std::vector<std::string> getDependents(const std::string& key) const;
    bool hasDependency(const std::string& from, const std::string& to) const;

    void addObserver(std::shared_ptr<GroupDomainObserver> observer);
    void removeObserver(std::shared_ptr<GroupDomainObserver> observer);
    void notifyObservers(const GroupEvent& event);

    const std::string& getName() const { return group_name; }
    const GroupDomainConfig& getConfig() const { return config; }
    GroupDomainConfig& getConfig() { return config; }

    bool hasConverged() const { return converged; }
    void setConverged(bool value) { converged = value; }

    size_t getIterationCount() const { return iteration_count; }
    void incrementIteration() { ++iteration_count; }

    bool operator==(const GroupDomain& other) const;
    bool operator<=(const GroupDomain& other) const;

private:
    std::string group_name;
    GroupDomainConfig config;
    std::map<std::string, std::unique_ptr<AbstractDomain>> members;
    std::map<std::string, std::set<std::string>> dependency_graph;
    std::vector<std::shared_ptr<GroupDomainObserver>> observers;
    bool converged;
    size_t iteration_count;
    bool is_top;
    bool is_bottom;
};

class GroupDomainImpl final : public GroupDomain {
public:
    using GroupDomain::GroupDomain;
};

}
#endif