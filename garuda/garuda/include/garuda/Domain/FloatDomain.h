#ifndef GARUDA_FLOAT_DOMAIN_H
#define GARUDA_FLOAT_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <llvm/ADT/APFloat.h>

namespace garuda {

class FloatDomain final : public AbstractDomain {
public:
    FloatDomain();
    explicit FloatDomain(double lo, double hi);
    explicit FloatDomain(double value);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::FLOAT; }

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

    double getLowerBound() const { return lower_bound; }
    double getUpperBound() const { return upper_bound; }
    void setBounds(double lo, double hi) {
        lower_bound = lo;
        upper_bound = hi;
        is_top = false;
        is_bottom = false;
    }

    bool operator==(const FloatDomain& other) const;
    bool operator<=(const FloatDomain& other) const;

private:
    double lower_bound;
    double upper_bound;
    bool is_top;
    bool is_bottom;
};

}
#endif