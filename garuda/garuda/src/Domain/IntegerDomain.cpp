#include "garuda/Domain/IntegerDomain.h"
#include "garuda/Domain/BooleanDomain.h"

namespace garuda {

IntegerDomain::IntegerDomain()
    : lower_bound(INT64_MIN), upper_bound(INT64_MAX), is_top(true), is_bottom(false) {}

IntegerDomain::IntegerDomain(int64_t lo, int64_t hi)
    : lower_bound(lo), upper_bound(hi), is_top(false), is_bottom(false) {}

IntegerDomain::IntegerDomain(int64_t value)
    : lower_bound(value), upper_bound(value), is_top(false), is_bottom(false) {}

std::unique_ptr<AbstractDomain> IntegerDomain::clone() const {
    return std::make_unique<IntegerDomain>(*this);
}

bool IntegerDomain::isTop() const { return is_top; }
bool IntegerDomain::isBottom() const { return is_bottom; }

bool IntegerDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntegerDomain*>(&other)) {
        return *this == *other_int;
    }
    return false;
}

bool IntegerDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntegerDomain*>(&other)) {
        return *this <= *other_int;
    }
    return false;
}

std::unique_ptr<AbstractDomain> IntegerDomain::join(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntegerDomain*>(&other)) {
        auto result = std::make_unique<IntegerDomain>(
            std::min(lower_bound, other_int->lower_bound),
            std::max(upper_bound, other_int->upper_bound));
        return result;
    }
    return clone();
}

std::unique_ptr<AbstractDomain> IntegerDomain::meet(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntegerDomain*>(&other)) {
        auto result = std::make_unique<IntegerDomain>(
            std::max(lower_bound, other_int->lower_bound),
            std::min(upper_bound, other_int->upper_bound));
        if (result->lower_bound > result->upper_bound) {
            result->setBottom();
        }
        return result;
    }
    return clone();
}

std::unique_ptr<AbstractDomain> IntegerDomain::widen(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntegerDomain*>(&other)) {
        int64_t new_lo = (other_int->lower_bound < lower_bound) ? INT64_MIN : lower_bound;
        int64_t new_hi = (other_int->upper_bound > upper_bound) ? INT64_MAX : upper_bound;
        return std::make_unique<IntegerDomain>(new_lo, new_hi);
    }
    return clone();
}

std::unique_ptr<AbstractDomain> IntegerDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> IntegerDomain::applyUnaryOp(DomainOperator op) const {
    return clone();
}

std::unique_ptr<AbstractDomain> IntegerDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    return clone();
}

std::unique_ptr<AbstractDomain> IntegerDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    return std::make_unique<BooleanDomain>(false);
}

void IntegerDomain::print(llvm::raw_ostream& os) const {
    os << toString();
}

std::string IntegerDomain::toString() const {
    if (is_top) return "TOP";
    if (is_bottom) return "BOTTOM";
    if (isSingleton()) return "[" + std::to_string(lower_bound) + "]";
    return "[" + std::to_string(lower_bound) + ", " + std::to_string(upper_bound) + "]";
}

DomainState IntegerDomain::getState() const {
    DomainState s;
    s.isTop = is_top;
    s.isBottom = is_bottom;
    s.type = DomainType::INTEGER;
    return s;
}

void IntegerDomain::setState(const DomainState& state) {
    is_top = state.isTop;
    is_bottom = state.isBottom;
}

std::vector<const AbstractDomain*> IntegerDomain::getSubdomains() const {
    return {};
}

void IntegerDomain::addSubdomain(std::unique_ptr<AbstractDomain> sub) {}

llvm::APSInt IntegerDomain::getAbstractInteger() const {
    if (isSingleton()) return llvm::APSInt(lower_bound, true);
    return llvm::APSInt();
}

llvm::APFloat IntegerDomain::getAbstractFloat() const {
    if (isSingleton()) return llvm::APFloat(static_cast<double>(lower_bound));
    return llvm::APFloat(0.0);
}

bool IntegerDomain::getAbstractBoolean() const {
    return false;
}

void IntegerDomain::setTop() {
    is_top = true;
    is_bottom = false;
}

void IntegerDomain::setBottom() {
    is_top = false;
    is_bottom = true;
}

size_t IntegerDomain::getHash() const {
    return std::hash<std::string>{}(toString());
}

bool IntegerDomain::operator==(const IntegerDomain& other) const {
    return is_top == other.is_top && is_bottom == other.is_bottom &&
           (is_top || is_bottom || (lower_bound == other.lower_bound && upper_bound == other.upper_bound));
}

bool IntegerDomain::operator<=(const IntegerDomain& other) const {
    if (is_top || other.is_bottom) return true;
    if (other.is_top || is_bottom) return false;
    return lower_bound >= other.lower_bound && upper_bound <= other.upper_bound;
}

}