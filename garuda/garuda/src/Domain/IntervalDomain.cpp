#include "garuda/Domain/IntervalDomain.h"
#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include <llvm/Support/raw_ostream.h>
#include <algorithm>
#include <cmath>

namespace garuda {

IntervalDomain::IntervalDomain()
    : lower(0), upper(0), is_top(true), is_bottom(false) {
    state.type = DomainType::INTERVAL;
    state.isTop = true;
}

IntervalDomain::IntervalDomain(int64_t lo, int64_t hi)
    : lower(lo), upper(hi), is_top(false), is_bottom(false) {
    state.type = DomainType::INTERVAL;
    state.isTop = false;
    state.isBottom = false;
}

IntervalDomain::IntervalDomain(int64_t value)
    : lower(value), upper(value), is_top(false), is_bottom(false) {
    state.type = DomainType::INTERVAL;
    state.isTop = false;
    state.isBottom = false;
}

std::unique_ptr<AbstractDomain> IntervalDomain::clone() const {
    return std::make_unique<IntervalDomain>(*this);
}

bool IntervalDomain::isTop() const { return is_top; }
bool IntervalDomain::isBottom() const { return is_bottom; }

bool IntervalDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        return lower == other_int->lower && upper == other_int->upper &&
               is_top == other_int->is_top && is_bottom == other_int->is_bottom;
    }
    return false;
}

bool IntervalDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        if (is_top || other_int->is_bottom) return true;
        if (is_bottom || other_int->is_top) return false;
        return lower >= other_int->lower && upper <= other_int->upper;
    }
    return false;
}

std::unique_ptr<AbstractDomain> IntervalDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return clone();
    if (is_bottom) return other.clone();
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        auto result = std::make_unique<IntervalDomain>(
            std::min(lower, other_int->lower),
            std::max(upper, other_int->upper)
        );
        return result;
    }
    auto result = std::make_unique<IntervalDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> IntervalDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_bottom || other.isTop()) return clone();
    if (is_top) return other.clone();
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        int64_t new_lo = std::max(lower, other_int->lower);
        int64_t new_hi = std::min(upper, other_int->upper);
        if (new_lo > new_hi) {
            auto result = std::make_unique<IntervalDomain>();
            result->setBottom();
            return result;
        }
        return std::make_unique<IntervalDomain>(new_lo, new_hi);
    }
    auto result = std::make_unique<IntervalDomain>();
    result->setBottom();
    return result;
}

std::unique_ptr<AbstractDomain> IntervalDomain::widen(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return other.clone();
    if (is_bottom) return clone();
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        int64_t new_lo = lower < other_int->lower ? INT64_MIN : other_int->lower;
        int64_t new_hi = upper > other_int->upper ? INT64_MAX : other_int->upper;
        return std::make_unique<IntervalDomain>(new_lo, new_hi);
    }
    auto result = std::make_unique<IntervalDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> IntervalDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> IntervalDomain::applyUnaryOp(DomainOperator op) const {
    switch (op) {
        case DomainOperator::SUB:
        case DomainOperator::NEG:
            return std::make_unique<IntervalDomain>(-upper, -lower);
        default:
            return clone();
    }
}

std::unique_ptr<AbstractDomain> IntervalDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        switch (op) {
            case DomainOperator::ADD:
                return std::make_unique<IntervalDomain>(
                    lower + other_int->lower,
                    upper + other_int->upper
                );
            case DomainOperator::SUB:
                return std::make_unique<IntervalDomain>(
                    lower - other_int->upper,
                    upper - other_int->lower
                );
            case DomainOperator::MUL: {
                int64_t a = lower * other_int->lower;
                int64_t b = lower * other_int->upper;
                int64_t c = upper * other_int->lower;
                int64_t d = upper * other_int->upper;
                int64_t new_lo = std::min({a, b, c, d});
                int64_t new_hi = std::max({a, b, c, d});
                return std::make_unique<IntervalDomain>(new_lo, new_hi);
            }
            case DomainOperator::DIV:
                if (other_int->lower <= 0 && other_int->upper >= 0) {
                    auto result = std::make_unique<IntervalDomain>();
                    result->setTop();
                    return result;
                }
                return std::make_unique<IntervalDomain>(
                    lower / other_int->upper,
                    upper / other_int->lower
                );
            default:
                break;
        }
    }
    return clone();
}

std::unique_ptr<AbstractDomain> IntervalDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    if (auto* other_int = dynamic_cast<const IntervalDomain*>(&other)) {
        bool result = false;
        switch (cmp) {
            case DomainOperator::COMPARE_LT:
                result = upper < other_int->lower;
                break;
            case DomainOperator::COMPARE_LE:
                result = upper <= other_int->lower;
                break;
            case DomainOperator::COMPARE_GT:
                result = lower > other_int->upper;
                break;
            case DomainOperator::COMPARE_GE:
                result = lower >= other_int->upper;
                break;
            default:
                break;
        }
        return std::make_unique<BooleanDomain>(result);
    }
    return std::make_unique<BooleanDomain>(false);
}

void IntervalDomain::print(llvm::raw_ostream& os) const {
    if (is_top) os << "TOP";
    else if (is_bottom) os << "BOTTOM";
    else if (lower == upper) os << lower;
    else os << "[" << lower << ", " << upper << "]";
}

std::string IntervalDomain::toString() const {
    if (is_top) return "TOP";
    if (is_bottom) return "BOTTOM";
    if (lower == upper) return std::to_string(lower);
    return "[" + std::to_string(lower) + ", " + std::to_string(upper) + "]";
}

DomainState IntervalDomain::getState() const { return state; }
void IntervalDomain::setState(const DomainState& new_state) { state = new_state; }

llvm::APSInt IntervalDomain::getAbstractInteger() const {
    if (is_top) return llvm::APSInt();
    return llvm::APSInt(lower);
}

llvm::APFloat IntervalDomain::getAbstractFloat() const {
    return llvm::APFloat(static_cast<double>(lower));
}

bool IntervalDomain::getAbstractBoolean() const {
    return lower != 0 || upper != 0;
}

void IntervalDomain::setTop() {
    is_top = true;
    is_bottom = false;
    state.isTop = true;
    state.isBottom = false;
}

void IntervalDomain::setBottom() {
    is_top = false;
    is_bottom = true;
    state.isTop = false;
    state.isBottom = true;
}

size_t IntervalDomain::getHash() const {
    size_t h = std::hash<int64_t>{}(lower);
    h ^= std::hash<int64_t>{}(upper) + 0x9e3779b9 + (h << 6) + (h >> 2);
    return h;
}

bool IntervalDomain::overlaps(const IntervalDomain& other) const {
    return lower <= other.upper && upper >= other.lower;
}

bool IntervalDomain::containsInterval(int64_t lo, int64_t hi) const {
    return lower <= lo && upper >= hi;
}

IntervalDomain IntervalDomain::intersect(const IntervalDomain& other) const {
    int64_t new_lo = std::max(lower, other.lower);
    int64_t new_hi = std::min(upper, other.upper);
    if (new_lo > new_hi) return IntervalDomain(0, 0);
    return IntervalDomain(new_lo, new_hi);
}

IntervalDomain IntervalDomain::union_(const IntervalDomain& other) const {
    return IntervalDomain(
        std::min(lower, other.lower),
        std::max(upper, other.upper)
    );
}

bool IntervalDomain::operator==(const IntervalDomain& other) const {
    return lower == other.lower && upper == other.upper &&
           is_top == other.is_top && is_bottom == other.is_bottom;
}

bool IntervalDomain::operator<=(const IntervalDomain& other) const {
    return isLessThanOrEqual(other);
}

}