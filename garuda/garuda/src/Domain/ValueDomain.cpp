#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/IntegerDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include "garuda/Domain/PointerDomain.h"
#include "garuda/Domain/FloatDomain.h"
#include "garuda/Domain/IntervalDomain.h"
#include "garuda/Domain/ConstantDomain.h"
#include <llvm/IR/Instructions.h>
#include <llvm/Support/raw_ostream.h>
#include <cassert>

namespace garuda {

class ValueDomain final : public AbstractDomain {
public:
    ValueDomain() = default;
    explicit ValueDomain(const std::string& val) : value(val) {}

    std::unique_ptr<AbstractDomain> clone() const override {
        return std::make_unique<ValueDomain>(*this);
    }

    DomainType getDomainType() const override { return DomainType::TOP; }

    bool isTop() const override { return value.empty() && !is_bottom; }
    bool isBottom() const override { return is_bottom; }

    bool isEqual(const AbstractDomain& other) const override {
        if (auto* v = dynamic_cast<const ValueDomain*>(&other)) {
            return value == v->value && is_bottom == v->is_bottom;
        }
        return false;
    }

    bool isLessThanOrEqual(const AbstractDomain& other) const override {
        if (other.isTop()) return true;
        if (isBottom()) return true;
        if (other.isBottom()) return false;
        return isEqual(other);
    }

    std::unique_ptr<AbstractDomain> join(const AbstractDomain& other) const override {
        if (isEqual(other)) return clone();
        if (isTop() || other.isBottom()) return clone();
        if (isBottom()) return other.clone();
        auto result = std::make_unique<ValueDomain>();
        result->setTop();
        return result;
    }

    std::unique_ptr<AbstractDomain> meet(const AbstractDomain& other) const override {
        if (isEqual(other)) return clone();
        if (isBottom() || other.isTop()) return clone();
        if (isTop()) return other.clone();
        auto result = std::make_unique<ValueDomain>();
        result->setBottom();
        return result;
    }

    std::unique_ptr<AbstractDomain> widen(const AbstractDomain& other) const override {
        return join(other);
    }

    std::unique_ptr<AbstractDomain> narrow(const AbstractDomain& other) const override {
        return meet(other);
    }

    std::unique_ptr<AbstractDomain> applyUnaryOp(DomainOperator op) const override {
        return clone();
    }

    std::unique_ptr<AbstractDomain> applyBinaryOp(DomainOperator op, const AbstractDomain& other) const override {
        return clone();
    }

    std::unique_ptr<AbstractDomain> applyCompare(DomainOperator cmp, const AbstractDomain& other) const override {
        return std::make_unique<BooleanDomain>(false);
    }

    void print(llvm::raw_ostream& os) const override {
        if (isTop()) os << "TOP";
        else if (isBottom()) os << "BOTTOM";
        else os << "Value(" << value << ")";
    }

    std::string toString() const override {
        if (isTop()) return "TOP";
        if (isBottom()) return "BOTTOM";
        return "Value(" + value + ")";
    }

    DomainState getState() const override { return state; }
    void setState(const DomainState& new_state) override { state = new_state; }

    std::vector<const AbstractDomain*> getSubdomains() const override { return {}; }
    void addSubdomain(std::unique_ptr<AbstractDomain> sub) override {}

    llvm::APSInt getAbstractInteger() const override { return llvm::APSInt(); }
    llvm::APFloat getAbstractFloat() const override { 
        llvm::APFloat f(llvm::APFloat::IEEEdouble());
        return f;
    }
    bool getAbstractBoolean() const override { return false; }

    void setTop() override { state.isTop = true; state.isBottom = false; }
    void setBottom() override { state.isTop = false; state.isBottom = true; }

    size_t getHash() const override {
        return std::hash<std::string>{}(toString());
    }

    const std::string& getValue() const { return value; }
    void setValue(const std::string& v) { value = v; }

private:
    std::string value;
    bool is_bottom = false;
};

}