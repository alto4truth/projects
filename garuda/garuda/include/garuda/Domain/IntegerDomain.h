#ifndef GARUDA_INTEGER_DOMAIN_H
#define GARUDA_INTEGER_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <llvm/ADT/APSInt.h>

namespace garuda {

class IntegerDomain final : public AbstractDomain {
public:
    IntegerDomain();
    explicit IntegerDomain(int64_t lo, int64_t hi);
    explicit IntegerDomain(int64_t value);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::INTEGER; }

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

    std::vector<const AbstractDomain*> getSubdomains() const override;
    void addSubdomain(std::unique_ptr<AbstractDomain> sub) override;

    llvm::APSInt getAbstractInteger() const override;
    llvm::APFloat getAbstractFloat() const override;
    bool getAbstractBoolean() const override;

    void setTop() override;
    void setBottom() override;

    size_t getHash() const override;

    int64_t getLowerBound() const { return lower_bound; }
    int64_t getUpperBound() const { return upper_bound; }
    bool isSingleton() const { return lower_bound == upper_bound && !is_top && !is_bottom; }

    void setBounds(int64_t lo, int64_t hi) {
        lower_bound = lo;
        upper_bound = hi;
        is_top = false;
        is_bottom = false;
    }

    bool operator==(const IntegerDomain& other) const;
    bool operator<=(const IntegerDomain& other) const;

private:
    int64_t lower_bound;
    int64_t upper_bound;
    bool is_top;
    bool is_bottom;
};

}

#endif