#ifndef GARUDA_INTERVAL_DOMAIN_H
#define GARUDA_INTERVAL_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <llvm/ADT/APSInt.h>

namespace garuda {

class IntervalDomain final : public AbstractDomain {
public:
    IntervalDomain();
    explicit IntervalDomain(int64_t lo, int64_t hi);
    explicit IntervalDomain(int64_t value);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::INTERVAL; }

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

    int64_t getLower() const { return lower; }
    int64_t getUpper() const { return upper; }
    bool contains(int64_t val) const { return val >= lower && val <= upper; }
    bool isSingleton() const { return lower == upper; }
    bool overlaps(const IntervalDomain& other) const;
    bool containsInterval(int64_t lo, int64_t hi) const;

    IntervalDomain intersect(const IntervalDomain& other) const;
    IntervalDomain union_(const IntervalDomain& other) const;

    bool operator==(const IntervalDomain& other) const;
    bool operator<=(const IntervalDomain& other) const;

private:
    int64_t lower;
    int64_t upper;
    bool is_top;
    bool is_bottom;
};

}
#endif