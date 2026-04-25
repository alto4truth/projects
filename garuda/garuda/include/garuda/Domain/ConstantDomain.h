#ifndef GARUDA_CONSTANT_DOMAIN_H
#define GARUDA_CONSTANT_DOMAIN_H

#include "garuda/Domain/AbstractDomain.h"
#include <llvm/ADT/APSInt.h>
#include <llvm/ADT/APFloat.h>

namespace garuda {

class ConstantDomain final : public AbstractDomain {
public:
    ConstantDomain();
    explicit ConstantDomain(int64_t value);
    explicit ConstantDomain(double value);
    explicit ConstantDomain(bool value);
    explicit ConstantDomain(const std::string& value);

    std::unique_ptr<AbstractDomain> clone() const override;
    DomainType getDomainType() const override { return DomainType::CONSTANT; }

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

    bool isInteger() const { return std::holds_alternative<int64_t>(value); }
    bool isFloat() const { return std::holds_alternative<double>(value); }
    bool isBoolean() const { return std::holds_alternative<bool>(value); }
    bool isString() const { return std::holds_alternative<std::string>(value); }

    int64_t getIntValue() const { return std::get<int64_t>(value); }
    double getFloatValue() const { return std::get<double>(value); }
    bool getBoolValue() const { return std::get<bool>(value); }
    const std::string& getStringValue() const { return std::get<std::string>(value); }

    bool operator==(const ConstantDomain& other) const;
    bool operator<=(const ConstantDomain& other) const;

private:
    std::variant<int64_t, double, bool, std::string> value;
};

}
#endif