#include "garuda/Domain/AbstractDomain.h"

namespace garuda {

std::unique_ptr<AbstractDomain> AbstractDomain::clone() const {
    return nullptr;
}

DomainType AbstractDomain::getDomainType() const {
    return state.type;
}

bool AbstractDomain::isTop() const {
    return state.isTop;
}

bool AbstractDomain::isBottom() const {
    return state.isBottom;
}

bool AbstractDomain::isEqual(const AbstractDomain& other) const {
    if (getDomainType() != other.getDomainType()) return false;
    return toString() == other.toString();
}

bool AbstractDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    auto joined = join(other);
    return joined->isEqual(other);
}

std::unique_ptr<AbstractDomain> AbstractDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (isTop() || other.isBottom()) return clone();
    if (isBottom()) return other.clone();
    return std::unique_ptr<AbstractDomain>(nullptr);
}

std::unique_ptr<AbstractDomain> AbstractDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (isBottom() || other.isTop()) return clone();
    if (isTop()) return other.clone();
    return std::unique_ptr<AbstractDomain>(nullptr);
}

std::unique_ptr<AbstractDomain> AbstractDomain::widen(const AbstractDomain& other) const {
    return join(other);
}

std::unique_ptr<AbstractDomain> AbstractDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> AbstractDomain::applyUnaryOp(DomainOperator op) const {
    return clone();
}

std::unique_ptr<AbstractDomain> AbstractDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    return clone();
}

std::unique_ptr<AbstractDomain> AbstractDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    return nullptr;
}

void AbstractDomain::print(llvm::raw_ostream& os) const {
    os << toString();
}

std::string AbstractDomain::toString() const {
    if (isTop()) return "TOP";
    if (isBottom()) return "BOTTOM";
    return "DOMAIN";
}

DomainState AbstractDomain::getState() const {
    return state;
}

void AbstractDomain::setState(const DomainState& new_state) {
    state = new_state;
}

std::vector<const AbstractDomain*> AbstractDomain::getSubdomains() const {
    return {};
}

void AbstractDomain::addSubdomain(std::unique_ptr<AbstractDomain> sub) {}

llvm::APSInt AbstractDomain::getAbstractInteger() const {
    return llvm::APSInt();
}

llvm::APFloat AbstractDomain::getAbstractFloat() const {
    static llvm::APFloat f(0.0);
    return f;
}

bool AbstractDomain::getAbstractBoolean() const {
    return false;
}

void AbstractDomain::setTop() {
    state.isTop = true;
    state.isBottom = false;
}

void AbstractDomain::setBottom() {
    state.isTop = false;
    state.isBottom = true;
}

size_t AbstractDomain::getHash() const {
    return std::hash<std::string>{}(toString());
}

}