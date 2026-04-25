#ifndef GARUDA_POINTER_DOMAIN_H
#define GARUDA_POINTER_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <cstdint>

namespace garuda {

class PointerDomain final : public AbstractDomain {
public:
    PointerDomain();
    explicit PointerDomain(uint64_t address);
    explicit PointerDomain(const std::string& allocation_site);
    explicit PointerDomain(uint64_t address, const std::string& alloc_site);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::POINTER; }

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

    uint64_t getAddress() const { return address; }
    const std::string& getAllocationSite() const { return allocation_site; }
    bool isNull() const { return address == 0; }
    bool pointsToHeap() const { return allocation_site == "heap"; }
    bool pointsToStack() const { return allocation_site == "stack"; }
    bool pointsToGlobal() const { return allocation_site == "global"; }

    bool operator==(const PointerDomain& other) const;
    bool operator<=(const PointerDomain& other) const;

private:
    uint64_t address;
    std::string allocation_site;
    bool is_top;
    bool is_bottom;
};

}
#endif