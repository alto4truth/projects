#include "garuda/Group/GroupDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include <llvm/Support/raw_ostream.h>
#include <sstream>

namespace garuda {

GroupDomain::GroupDomain()
    : group_name("default"),
      config(GroupDomainConfig()),
      converged(false),
      iteration_count(0),
      is_top(true),
      is_bottom(false) {
    state.type = DomainType::GROUP;
    state.isTop = true;
}

GroupDomain::GroupDomain(const std::string& name)
    : group_name(name),
      config(GroupDomainConfig()),
      converged(false),
      iteration_count(0),
      is_top(false),
      is_bottom(false) {
    state.type = DomainType::GROUP;
    state.isTop = false;
    state.isBottom = false;
}

GroupDomain::GroupDomain(const std::string& name, const GroupDomainConfig& cfg)
    : group_name(name),
      config(cfg),
      converged(false),
      iteration_count(0),
      is_top(false),
      is_bottom(false) {
    state.type = DomainType::GROUP;
    state.isTop = false;
    state.isBottom = false;
}

std::unique_ptr<AbstractDomain> GroupDomain::clone() const {
    auto result = std::make_unique<GroupDomain>(group_name, config);
    result->is_top = is_top;
    result->is_bottom = is_bottom;
    result->converged = converged;
    result->iteration_count = iteration_count;
    result->state = state;
    for (const auto& [key, dom] : members) {
        result->members[key] = dom->clone();
    }
    result->dependency_graph = dependency_graph;
    return result;
}

bool GroupDomain::isTop() const { return is_top; }
bool GroupDomain::isBottom() const { return is_bottom; }

bool GroupDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        if (group_name != other_group->group_name) return false;
        if (members.size() != other_group->members.size()) return false;
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it == other_group->members.end()) return false;
            if (!pair.second->isEqual(*it->second)) return false;
        }
        return true;
    }
    return false;
}

bool GroupDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        if (is_top || other_group->is_bottom) return true;
        if (is_bottom || other_group->is_top) return false;
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it == other_group->members.end()) return false;
            if (!pair.second->isLessThanOrEqual(*it->second)) return false;
        }
        return true;
    }
    return false;
}

std::unique_ptr<AbstractDomain> GroupDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return clone();
    if (is_bottom) return other.clone();
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        auto result = std::make_unique<GroupDomain>(group_name, config);
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it != other_group->members.end()) {
                auto joined = pair.second->join(*it->second);
                result->addMember(pair.first, std::move(joined));
            } else {
                result->addMember(pair.first, pair.second->clone());
            }
        }
        for (const auto& pair : other_group->members) {
            if (members.find(pair.first) == members.end()) {
                result->addMember(pair.first, pair.second->clone());
            }
        }
        return result;
    }
    auto result = std::make_unique<GroupDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> GroupDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_bottom || other.isTop()) return clone();
    if (is_top) return other.clone();
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        auto result = std::make_unique<GroupDomain>(group_name, config);
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it != other_group->members.end()) {
                auto met = pair.second->meet(*it->second);
                result->addMember(pair.first, std::move(met));
            }
        }
        return result;
    }
    auto result = std::make_unique<GroupDomain>();
    result->setBottom();
    return result;
}

std::unique_ptr<AbstractDomain> GroupDomain::widen(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return other.clone();
    if (is_bottom) return clone();
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        auto result = std::make_unique<GroupDomain>(group_name, config);
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it != other_group->members.end()) {
                auto widened = pair.second->widen(*it->second);
                result->addMember(pair.first, std::move(widened));
            }
        }
        return result;
    }
    auto result = std::make_unique<GroupDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> GroupDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> GroupDomain::applyUnaryOp(DomainOperator op) const {
    auto result = std::make_unique<GroupDomain>(group_name, config);
    for (const auto& pair : members) {
        auto transformed = pair.second->applyUnaryOp(op);
        result->addMember(pair.first, std::move(transformed));
    }
    return result;
}

std::unique_ptr<AbstractDomain> GroupDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    if (auto* other_group = dynamic_cast<const GroupDomain*>(&other)) {
        auto result = std::make_unique<GroupDomain>(group_name, config);
        for (const auto& pair : members) {
            auto it = other_group->members.find(pair.first);
            if (it != other_group->members.end()) {
                auto transformed = pair.second->applyBinaryOp(op, *it->second);
                result->addMember(pair.first, std::move(transformed));
            }
        }
        return result;
    }
    return clone();
}

std::unique_ptr<AbstractDomain> GroupDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    return std::make_unique<BooleanDomain>(false);
}

void GroupDomain::print(llvm::raw_ostream& os) const {
    if (is_top) { os << "TOP"; return; }
    if (is_bottom) { os << "BOTTOM"; return; }
    os << "Group(" << group_name << "): {";
    bool first = true;
    for (const auto& pair : members) {
        if (!first) os << ", ";
        os << pair.first << " -> " << *pair.second;
        first = false;
    }
    os << "}";
}

std::string GroupDomain::toString() const {
    if (is_top) return "TOP";
    if (is_bottom) return "BOTTOM";
    std::ostringstream oss;
    oss << "Group(" << group_name << "): {";
    bool first = true;
    for (const auto& pair : members) {
        if (!first) oss << ", ";
        oss << pair.first << " -> " << pair.second->toString();
        first = false;
    }
    oss << "}";
    return oss.str();
}

DomainState GroupDomain::getState() const { return state; }
void GroupDomain::setState(const DomainState& new_state) { state = new_state; }

llvm::APSInt GroupDomain::getAbstractInteger() const { return llvm::APSInt(); }
llvm::APFloat GroupDomain::getAbstractFloat() const { 
    static llvm::APFloat f(0.0);
    return f;
}
bool GroupDomain::getAbstractBoolean() const { return !members.empty(); }

void GroupDomain::setTop() {
    is_top = true;
    is_bottom = false;
    state.isTop = true;
    state.isBottom = false;
}

void GroupDomain::setBottom() {
    is_top = false;
    is_bottom = true;
    state.isTop = false;
    state.isBottom = true;
}

size_t GroupDomain::getHash() const {
    size_t h = std::hash<std::string>{}(group_name);
    for (const auto& pair : members) {
        h ^= pair.second->getHash() + 0x9e3779b9 + (h << 6) + (h >> 2);
    }
    return h;
}

void GroupDomain::addMember(const std::string& key, std::unique_ptr<AbstractDomain> domain) {
    members[key] = std::move(domain);
    is_top = false;
}

std::unique_ptr<AbstractDomain> GroupDomain::getMember(const std::string& key) const {
    auto it = members.find(key);
    if (it != members.end()) {
        return it->second->clone();
    }
    return nullptr;
}

bool GroupDomain::hasMember(const std::string& key) const {
    return members.find(key) != members.end();
}

void GroupDomain::removeMember(const std::string& key) {
    members.erase(key);
}

std::vector<std::string> GroupDomain::getMemberKeys() const {
    std::vector<std::string> keys;
    for (const auto& pair : members) {
        keys.push_back(pair.first);
    }
    return keys;
}

size_t GroupDomain::getMemberCount() const {
    return members.size();
}

void GroupDomain::addDependency(const std::string& from, const std::string& to) {
    dependency_graph[from].insert(to);
}

std::vector<std::string> GroupDomain::getDependencies(const std::string& key) const {
    std::vector<std::string> deps;
    auto it = dependency_graph.find(key);
    if (it != dependency_graph.end()) {
        for (const auto& dep : it->second) {
            deps.push_back(dep);
        }
    }
    return deps;
}

std::vector<std::string> GroupDomain::getDependents(const std::string& key) const {
    std::vector<std::string> deps;
    for (const auto& pair : dependency_graph) {
        if (pair.second.find(key) != pair.second.end()) {
            deps.push_back(pair.first);
        }
    }
    return deps;
}

bool GroupDomain::hasDependency(const std::string& from, const std::string& to) const {
    auto it = dependency_graph.find(from);
    return it != dependency_graph.end() && it->second.find(to) != it->second.end();
}

void GroupDomain::addObserver(std::shared_ptr<GroupDomainObserver> observer) {
    observers.push_back(observer);
}

void GroupDomain::removeObserver(std::shared_ptr<GroupDomainObserver> observer) {
    for (auto it = observers.begin(); it != observers.end(); ++it) {
        if (*it == observer) {
            observers.erase(it);
            break;
        }
    }
}

void GroupDomain::notifyObservers(const GroupEvent& event) {
    for (const auto& obs : observers) {
        obs->onGroupEvent(event);
    }
}

bool GroupDomain::operator==(const GroupDomain& other) const {
    return isEqual(other);
}

bool GroupDomain::operator<=(const GroupDomain& other) const {
    return isLessThanOrEqual(other);
}

}