#ifndef GARUDA_BOOLEAN_DOMAIN_H
#define GARUDA_BOOLEAN_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"

namespace garuda {

class BooleanDomain final : public AbstractDomain {
public:
    BooleanDomain();
    explicit BooleanDomain(bool value);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::BOOLEAN; }

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

    bool getValue() const { return value; }
    void setValue(bool v) { value = v; is_top = false; is_bottom = false; }

    bool operator==(const BooleanDomain& other) const;
    bool operator<=(const BooleanDomain& other) const;

private:
    bool value;
    bool is_top;
    bool is_bottom;
};

}
#endif