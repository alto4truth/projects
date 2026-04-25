#include "garuda/Domain/BooleanDomain.h"
#include "garuda/Domain/AbstractDomain.h"
#include <llvm/Support/raw_ostream.h>
#include <cassert>

namespace garuda {

BooleanDomain::BooleanDomain() : value(false), is_top(true), is_bottom(false) {
    state.type = DomainType::BOOLEAN;
}

BooleanDomain::BooleanDomain(bool val) : value(val), is_top(false), is_bottom(false) {
    state.type = DomainType::BOOLEAN;
    state.isTop = false;
    state.isBottom = false;
}

std::unique_ptr<AbstractDomain> BooleanDomain::clone() const {
    return std::make_unique<BooleanDomain>(*this);
}

bool BooleanDomain::isTop() const {
    return is_top;
}

bool BooleanDomain::isBottom() const {
    return is_bottom;
}

bool BooleanDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_bool = dynamic_cast<const BooleanDomain*>(&other)) {
        return value == other_bool->value && is_top == other_bool->is_top && is_bottom == other_bool->is_bottom;
    }
    return false;
}

bool BooleanDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_bool = dynamic_cast<const BooleanDomain*>(&other)) {
        if (is_top || other_bool->is_bottom) return true;
        if (is_bottom || other_bool->is_top) return false;
        return value == other_bool->value;
    }
    return false;
}

std::unique_ptr<AbstractDomain> BooleanDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return clone();
    if (is_bottom) return other.clone();
    auto result = std::make_unique<BooleanDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> BooleanDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_bottom || other.isTop()) return clone();
    if (is_top) return other.clone();
    auto result = std::make_unique<BooleanDomain>();
    result->setBottom();
    return result;
}

std::unique_ptr<AbstractDomain> BooleanDomain::widen(const AbstractDomain& other) const {
    return join(other);
}

std::unique_ptr<AbstractDomain> BooleanDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> BooleanDomain::applyUnaryOp(DomainOperator op) const {
    switch (op) {
        case DomainOperator::NOT:
            return std::make_unique<BooleanDomain>(!value);
        default:
            return clone();
    }
}

std::unique_ptr<AbstractDomain> BooleanDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    if (auto* other_bool = dynamic_cast<const BooleanDomain*>(&other)) {
        switch (op) {
            case DomainOperator::AND:
                return std::make_unique<BooleanDomain>(value && other_bool->value);
            case DomainOperator::OR:
                return std::make_unique<BooleanDomain>(value || other_bool->value);
            case DomainOperator::XOR:
                return std::make_unique<BooleanDomain>(value != other_bool->value);
            default:
                break;
        }
    }
    return clone();
}

std::unique_ptr<AbstractDomain> BooleanDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    if (auto* other_bool = dynamic_cast<const BooleanDomain*>(&other)) {
        switch (cmp) {
            case DomainOperator::COMPARE_EQ:
                return std::make_unique<BooleanDomain>(value == other_bool->value);
            case DomainOperator::COMPARE_NE:
                return std::make_unique<BooleanDomain>(value != other_bool->value);
            default:
                break;
        }
    }
    return std::make_unique<BooleanDomain>(false);
}

void BooleanDomain::print(llvm::raw_ostream& os) const {
    if (is_top) os << "TOP";
    else if (is_bottom) os << "BOTTOM";
    else os << (value ? "true" : "false");
}

std::string BooleanDomain::toString() const {
    if (is_top) return "TOP";
    if (is_bottom) return "BOTTOM";
    return value ? "true" : "false";
}

DomainState BooleanDomain::getState() const {
    return state;
}

void BooleanDomain::setState(const DomainState& new_state) {
    state = new_state;
    if (state.isTop) is_top = true;
    if (state.isBottom) is_bottom = true;
}

llvm::APSInt BooleanDomain::getAbstractInteger() const {
    return llvm::APSInt(value ? 1 : 0);
}

llvm::APFloat BooleanDomain::getAbstractFloat() const {
    return llvm::APFloat(value ? 1.0 : 0.0);
}

bool BooleanDomain::getAbstractBoolean() const {
    return value;
}

void BooleanDomain::setTop() {
    is_top = true;
    is_bottom = false;
    state.isTop = true;
    state.isBottom = false;
}

void BooleanDomain::setBottom() {
    is_top = false;
    is_bottom = true;
    state.isTop = false;
    state.isBottom = true;
}

size_t BooleanDomain::getHash() const {
    size_t h = std::hash<bool>{}(value);
    h ^= std::hash<bool>{}(is_top) + 0x9e3779b9 + (h << 6) + (h >> 2);
    h ^= std::hash<bool>{}(is_bottom) + 0x9e3779b9 + (h << 6) + (h >> 2);
    return h;
}

bool BooleanDomain::operator==(const BooleanDomain& other) const {
    return value == other.value && is_top == other.is_top && is_bottom == other.is_bottom;
}

bool BooleanDomain::operator<=(const BooleanDomain& other) const {
    return isLessThanOrEqual(other);
}

}