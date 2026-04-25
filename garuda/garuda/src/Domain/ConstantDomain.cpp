#include "garuda/Domain/ConstantDomain.h"
#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include <llvm/Support/raw_ostream.h>
#include <sstream>

namespace garuda {

ConstantDomain::ConstantDomain() : value(int64_t(0)) {
    state.type = DomainType::CONSTANT;
    state.isTop = false;
    state.isBottom = false;
}

ConstantDomain::ConstantDomain(int64_t val) : value(val) {
    state.type = DomainType::CONSTANT;
    state.isTop = false;
    state.isBottom = false;
}

ConstantDomain::ConstantDomain(double val) : value(val) {
    state.type = DomainType::CONSTANT;
    state.isTop = false;
    state.isBottom = false;
}

ConstantDomain::ConstantDomain(bool val) : value(val) {
    state.type = DomainType::CONSTANT;
    state.isTop = false;
    state.isBottom = false;
}

ConstantDomain::ConstantDomain(const std::string& val) : value(val) {
    state.type = DomainType::CONSTANT;
    state.isTop = false;
    state.isBottom = false;
}

std::unique_ptr<AbstractDomain> ConstantDomain::clone() const {
    return std::make_unique<ConstantDomain>(*this);
}

bool ConstantDomain::isTop() const {
    return state.isTop;
}

bool ConstantDomain::isBottom() const {
    return state.isBottom;
}

bool ConstantDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_const = dynamic_cast<const ConstantDomain*>(&other)) {
        return value == other_const->value;
    }
    return false;
}

bool ConstantDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_const = dynamic_cast<const ConstantDomain*>(&other)) {
        if (isInteger() && other_const->isInteger()) {
            return getIntValue() <= other_const->getIntValue();
        }
        if (isFloat() && other_const->isFloat()) {
            return getFloatValue() <= other_const->getFloatValue();
        }
    }
    return false;
}

std::unique_ptr<AbstractDomain> ConstantDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (isInteger() && std::holds_alternative<int64_t>(value)) {
        if (auto* other_const = dynamic_cast<const ConstantDomain*>(&other);
            other_const && other_const->isInteger()) {
            int64_t a = getIntValue();
            int64_t b = other_const->getIntValue();
            return std::make_unique<ConstantDomain>(std::min(a, b));
        }
    }
    auto result = std::make_unique<ConstantDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> ConstantDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    auto result = std::make_unique<ConstantDomain>();
    result->setBottom();
    return result;
}

std::unique_ptr<AbstractDomain> ConstantDomain::widen(const AbstractDomain& other) const {
    return join(other);
}

std::unique_ptr<AbstractDomain> ConstantDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> ConstantDomain::applyUnaryOp(DomainOperator op) const {
    switch (op) {
        case DomainOperator::NEG:
        case DomainOperator::SUB:
            if (isInteger()) {
                return std::make_unique<ConstantDomain>(-getIntValue());
            } else if (isFloat()) {
                return std::make_unique<ConstantDomain>(-getFloatValue());
            }
            break;
        case DomainOperator::NOT:
            if (isBoolean()) {
                return std::make_unique<ConstantDomain>(!getBoolValue());
            }
            break;
        default:
            break;
    }
    return clone();
}

std::unique_ptr<AbstractDomain> ConstantDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    if (auto* other_const = dynamic_cast<const ConstantDomain*>(&other)) {
        switch (op) {
            case DomainOperator::ADD:
                if (isInteger() && other_const->isInteger()) {
                    return std::make_unique<ConstantDomain>(getIntValue() + other_const->getIntValue());
                }
                if (isFloat() && other_const->isFloat()) {
                    return std::make_unique<ConstantDomain>(getFloatValue() + other_const->getFloatValue());
                }
                break;
            case DomainOperator::SUB:
                if (isInteger() && other_const->isInteger()) {
                    return std::make_unique<ConstantDomain>(getIntValue() - other_const->getIntValue());
                }
                if (isFloat() && other_const->isFloat()) {
                    return std::make_unique<ConstantDomain>(getFloatValue() - other_const->getFloatValue());
                }
                break;
            case DomainOperator::MUL:
                if (isInteger() && other_const->isInteger()) {
                    return std::make_unique<ConstantDomain>(getIntValue() * other_const->getIntValue());
                }
                if (isFloat() && other_const->isFloat()) {
                    return std::make_unique<ConstantDomain>(getFloatValue() * other_const->getFloatValue());
                }
                break;
            case DomainOperator::DIV:
                if (isInteger() && other_const->isInteger()) {
                    return std::make_unique<ConstantDomain>(getIntValue() / other_const->getIntValue());
                }
                if (isFloat() && other_const->isFloat()) {
                    return std::make_unique<ConstantDomain>(getFloatValue() / other_const->getFloatValue());
                }
                break;
            default:
                break;
        }
    }
    return clone();
}

std::unique_ptr<AbstractDomain> ConstantDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    if (auto* other_const = dynamic_cast<const ConstantDomain*>(&other)) {
        bool result = false;
        if (isInteger() && other_const->isInteger()) {
            switch (cmp) {
                case DomainOperator::COMPARE_EQ:
                    result = getIntValue() == other_const->getIntValue();
                    break;
                case DomainOperator::COMPARE_NE:
                    result = getIntValue() != other_const->getIntValue();
                    break;
                case DomainOperator::COMPARE_LT:
                    result = getIntValue() < other_const->getIntValue();
                    break;
                case DomainOperator::COMPARE_LE:
                    result = getIntValue() <= other_const->getIntValue();
                    break;
                case DomainOperator::COMPARE_GT:
                    result = getIntValue() > other_const->getIntValue();
                    break;
                case DomainOperator::COMPARE_GE:
                    result = getIntValue() >= other_const->getIntValue();
                    break;
                default:
                    break;
            }
        } else if (isFloat() && other_const->isFloat()) {
            switch (cmp) {
                case DomainOperator::COMPARE_EQ:
                    result = getFloatValue() == other_const->getFloatValue();
                    break;
                case DomainOperator::COMPARE_NE:
                    result = getFloatValue() != other_const->getFloatValue();
                    break;
                case DomainOperator::COMPARE_LT:
                    result = getFloatValue() < other_const->getFloatValue();
                    break;
                case DomainOperator::COMPARE_LE:
                    result = getFloatValue() <= other_const->getFloatValue();
                    break;
                case DomainOperator::COMPARE_GT:
                    result = getFloatValue() > other_const->getFloatValue();
                    break;
                case DomainOperator::COMPARE_GE:
                    result = getFloatValue() >= other_const->getFloatValue();
                    break;
                default:
                    break;
            }
        }
        return std::make_unique<BooleanDomain>(result);
    }
    return std::make_unique<BooleanDomain>(false);
}

void ConstantDomain::print(llvm::raw_ostream& os) const {
    if (isInteger()) os << getIntValue();
    else if (isFloat()) os << getFloatValue();
    else if (isBoolean()) os << (getBoolValue() ? "true" : "false");
    else if (isString()) os << "\"" << getStringValue() << "\"";
}

std::string ConstantDomain::toString() const {
    if (isInteger()) return std::to_string(getIntValue());
    if (isFloat()) return std::to_string(getFloatValue());
    if (isBoolean()) return getBoolValue() ? "true" : "false";
    if (isString()) return "\"" + getStringValue() + "\"";
    return "CONSTANT";
}

DomainState ConstantDomain::getState() const { return state; }
void ConstantDomain::setState(const DomainState& new_state) { state = new_state; }

llvm::APSInt ConstantDomain::getAbstractInteger() const {
    if (isInteger()) return llvm::APSInt(getIntValue());
    return llvm::APSInt();
}

llvm::APFloat ConstantDomain::getAbstractFloat() const {
    if (isFloat()) return llvm::APFloat(getFloatValue());
    if (isInteger()) return llvm::APFloat(static_cast<double>(getIntValue()));
    return llvm::APFloat(0.0);
}

bool ConstantDomain::getAbstractBoolean() const {
    if (isBoolean()) return getBoolValue();
    if (isInteger()) return getIntValue() != 0;
    if (isFloat()) return getFloatValue() != 0.0;
    return false;
}

void ConstantDomain::setTop() {
    state.isTop = true;
    state.isBottom = false;
}

void ConstantDomain::setBottom() {
    state.isTop = false;
    state.isBottom = true;
}

size_t ConstantDomain::getHash() const {
    if (isInteger()) return std::hash<int64_t>{}(getIntValue());
    if (isFloat()) return std::hash<double>{}(getFloatValue());
    if (isBoolean()) return std::hash<bool>{}(getBoolValue());
    if (isString()) return std::hash<std::string>{}(getStringValue());
    return 0;
}

bool ConstantDomain::operator==(const ConstantDomain& other) const {
    return value == other.value;
}

bool ConstantDomain::operator<=(const ConstantDomain& other) const {
    return isLessThanOrEqual(other);
}

}