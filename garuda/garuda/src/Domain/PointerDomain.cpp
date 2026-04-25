#include "garuda/Domain/PointerDomain.h"
#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include <llvm/Support/raw_ostream.h>

namespace garuda {

PointerDomain::PointerDomain()
    : address(0), allocation_site(""), is_top(true), is_bottom(false) {
    state.type = DomainType::POINTER;
    state.isTop = true;
}

PointerDomain::PointerDomain(uint64_t addr)
    : address(addr), allocation_site("unknown"), is_top(false), is_bottom(false) {
    state.type = DomainType::POINTER;
    state.isTop = false;
    state.isBottom = false;
}

PointerDomain::PointerDomain(const std::string& site)
    : address(0), allocation_site(site), is_top(false), is_bottom(false) {
    state.type = DomainType::POINTER;
    state.isTop = false;
    state.isBottom = false;
}

std::unique_ptr<AbstractDomain> PointerDomain::clone() const {
    return std::make_unique<PointerDomain>(*this);
}

bool PointerDomain::isTop() const { return is_top; }
bool PointerDomain::isBottom() const { return is_bottom; }

bool PointerDomain::isEqual(const AbstractDomain& other) const {
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        return address == other_ptr->address &&
               allocation_site == other_ptr->allocation_site &&
               is_top == other_ptr->is_top &&
               is_bottom == other_ptr->is_bottom;
    }
    return false;
}

bool PointerDomain::isLessThanOrEqual(const AbstractDomain& other) const {
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        if (is_top || other_ptr->is_bottom) return true;
        if (is_bottom || other_ptr->is_top) return false;
        return address == other_ptr->address && allocation_site == other_ptr->allocation_site;
    }
    return false;
}

std::unique_ptr<AbstractDomain> PointerDomain::join(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_top || other.isBottom()) return clone();
    if (is_bottom) return other.clone();
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        if (address == other_ptr->address && allocation_site == other_ptr->allocation_site) {
            return clone();
        }
    }
    auto result = std::make_unique<PointerDomain>();
    result->setTop();
    return result;
}

std::unique_ptr<AbstractDomain> PointerDomain::meet(const AbstractDomain& other) const {
    if (isEqual(other)) return clone();
    if (is_bottom || other.isTop()) return clone();
    if (is_top) return other.clone();
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        if (address == other_ptr->address && allocation_site == other_ptr->allocation_site) {
            return clone();
        }
    }
    auto result = std::make_unique<PointerDomain>();
    result->setBottom();
    return result;
}

std::unique_ptr<AbstractDomain> PointerDomain::widen(const AbstractDomain& other) const {
    return join(other);
}

std::unique_ptr<AbstractDomain> PointerDomain::narrow(const AbstractDomain& other) const {
    return meet(other);
}

std::unique_ptr<AbstractDomain> PointerDomain::applyUnaryOp(DomainOperator op) const {
    switch (op) {
        case DomainOperator::LOAD:
            return clone();
        default:
            return clone();
    }
}

std::unique_ptr<AbstractDomain> PointerDomain::applyBinaryOp(DomainOperator op, const AbstractDomain& other) const {
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        switch (op) {
            case DomainOperator::ADD:
            case DomainOperator::GEP:
                return std::make_unique<PointerDomain>(
                    address + other_ptr->address,
                    allocation_site + "+" + other_ptr->allocation_site
                );
            default:
                break;
        }
    }
    return clone();
}

std::unique_ptr<AbstractDomain> PointerDomain::applyCompare(DomainOperator cmp, const AbstractDomain& other) const {
    if (auto* other_ptr = dynamic_cast<const PointerDomain*>(&other)) {
        bool result = false;
        switch (cmp) {
            case DomainOperator::COMPARE_EQ:
                result = address == other_ptr->address;
                break;
            case DomainOperator::COMPARE_NE:
                result = address != other_ptr->address;
                break;
            default:
                break;
        }
        return std::make_unique<BooleanDomain>(result);
    }
    return std::make_unique<BooleanDomain>(false);
}

void PointerDomain::print(llvm::raw_ostream& os) const {
    if (is_top) os << "TOP";
    else if (is_bottom) os << "BOTTOM";
    else if (isNull()) os << "null";
    else os << "ptr(" << address << ", " << allocation_site << ")";
}

std::string PointerDomain::toString() const {
    if (is_top) return "TOP";
    if (is_bottom) return "BOTTOM";
    if (isNull()) return "null";
    return "ptr(" + std::to_string(address) + ", " + allocation_site + ")";
}

DomainState PointerDomain::getState() const { return state; }
void PointerDomain::setState(const DomainState& new_state) { state = new_state; }

llvm::APSInt PointerDomain::getAbstractInteger() const {
    return llvm::APSInt(address);
}

llvm::APFloat PointerDomain::getAbstractFloat() const {
    return llvm::APFloat(static_cast<double>(address));
}

bool PointerDomain::getAbstractBoolean() const {
    return address != 0;
}

void PointerDomain::setTop() {
    is_top = true;
    is_bottom = false;
    state.isTop = true;
    state.isBottom = false;
}

void PointerDomain::setBottom() {
    is_top = false;
    is_bottom = true;
    state.isTop = false;
    state.isBottom = true;
}

size_t PointerDomain::getHash() const {
    size_t h = std::hash<uint64_t>{}(address);
    h ^= std::hash<std::string>{}(allocation_site) + 0x9e3779b9 + (h << 6) + (h >> 2);
    return h;
}

bool PointerDomain::operator==(const PointerDomain& other) const {
    return address == other.address && allocation_site == other.allocation_site &&
           is_top == other.is_top && is_bottom == other.is_bottom;
}

bool PointerDomain::operator<=(const PointerDomain& other) const {
    return isLessThanOrEqual(other);
}

}