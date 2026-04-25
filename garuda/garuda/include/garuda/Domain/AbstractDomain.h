#ifndef GARUDA_ABSTRACT_DOMAIN_H
#define GARUDA_ABSTRACT_DOMAIN_H

#include <llvm/IR/Value.h>
#include <llvm/ADT/APSInt.h>
#include <llvm/ADT/APFloat.h>
#include <memory>
#include <vector>
#include <map>
#include <set>
#include <functional>
#include <optional>
#include <variant>

namespace garuda {

enum class DomainType {
    TOP,
    BOTTOM,
    INTEGER,
    POINTER,
    BOOLEAN,
    FLOAT,
    INTERVAL,
    CONSTANT,
    STRUCT,
    ARRAY,
    GROUP
};

enum class DomainOperator {
    JOIN,
    MEET,
    WIDEN,
    NARROW,
    ASSIGN,
    COMPARE_EQ,
    COMPARE_NE,
    COMPARE_LT,
    COMPARE_LE,
    COMPARE_GT,
    COMPARE_GE,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    AND,
    OR,
    XOR,
    NOT,
    NEG,
    SHL,
    SHR,
    ALLOC,
    LOAD,
    STORE,
    GEP,
    CALL,
    PHI,
    SELECT,
    CMP
};

struct DomainState {
    bool isTop;
    bool isBottom;
    DomainType type;
    std::map<std::string, std::variant<int64_t, uint64_t, double, bool, std::string>> constraints;
    std::vector<std::pair<DomainType, void*>> subtype_domains;

    DomainState() : isTop(true), isBottom(false), type(DomainType::TOP) {}

    bool isTotallyUnbounded() const { return isTop && !isBottom; }
    bool isConflicting() const { return isBottom; }
};

class AbstractDomain {
public:
    virtual ~AbstractDomain() = default;

    virtual std::unique_ptr<AbstractDomain> clone() const = 0;
    virtual DomainType getDomainType() const = 0;

    virtual bool isTop() const = 0;
    virtual bool isBottom() const = 0;
    virtual bool isEqual(const AbstractDomain& other) const = 0;
    virtual bool isLessThanOrEqual(const AbstractDomain& other) const = 0;

    virtual std::unique_ptr<AbstractDomain> join(const AbstractDomain& other) const = 0;
    virtual std::unique_ptr<AbstractDomain> meet(const AbstractDomain& other) const = 0;
    virtual std::unique_ptr<AbstractDomain> widen(const AbstractDomain& other) const = 0;
    virtual std::unique_ptr<AbstractDomain> narrow(const AbstractDomain& other) const = 0;

    virtual std::unique_ptr<AbstractDomain> applyUnaryOp(DomainOperator op) const = 0;
    virtual std::unique_ptr<AbstractDomain> applyBinaryOp(DomainOperator op, const AbstractDomain& other) const = 0;
    virtual std::unique_ptr<AbstractDomain> applyCompare(DomainOperator cmp, const AbstractDomain& other) const = 0;

    virtual void print(llvm::raw_ostream& os) const = 0;
    virtual std::string toString() const = 0;

    virtual DomainState getState() const = 0;
    virtual void setState(const DomainState& state) = 0;

    virtual std::vector<const AbstractDomain*> getSubdomains() const;
    virtual void addSubdomain(std::unique_ptr<AbstractDomain> sub);

    virtual llvm::APSInt getAbstractInteger() const = 0;
    virtual llvm::APFloat getAbstractFloat() const = 0;
    virtual bool getAbstractBoolean() const = 0;

    virtual void setTop() = 0;
    virtual void setBottom() = 0;

    virtual size_t getHash() const = 0;

protected:
    DomainState state;
};

inline llvm::raw_ostream& operator<<(llvm::raw_ostream& os, const AbstractDomain& domain) {
    domain.print(os);
    return os;
}

class DomainFactory {
public:
    static std::unique_ptr<AbstractDomain> createDomain(DomainType type);
    static std::unique_ptr<AbstractDomain> createTopDomain(DomainType type);
    static std::unique_ptr<AbstractDomain> createBottomDomain(DomainType type);
    static std::unique_ptr<AbstractDomain> createNonTaulDomain(DomainType type, int64_t value);
    static std::unique_ptr<AbstractDomain> createConstantIntegerDomain(int64_t value);
    static std::unique_ptr<AbstractDomain> createIntervalDomain(int64_t lo, int64_t hi);
    static std::unique_ptr<AbstractDomain> createGroupDomain(const std::string& group_name);
};

class IntegerDomain;
class BooleanDomain;
class PointerDomain;
class FloatDomain;
class IntervalDomain;
class ConstantDomain;
class GroupDomainImpl;

template<typename DomainImpl>
class DomainDecorator : public AbstractDomain {
public:
    std::unique_ptr<AbstractDomain> clone() const override {
        return std::make_unique<DomainImpl>(static_cast<const DomainImpl&>(*this));
    }

    bool isEqual(const AbstractDomain& other) const override {
        if (auto* other_impl = dynamic_cast<const DomainImpl*>(&other)) {
            return static_cast<const DomainImpl&>(*this) == *other_impl;
        }
        return false;
    }

    bool isLessThanOrEqual(const AbstractDomain& other) const override {
        if (auto* other_impl = dynamic_cast<const DomainImpl*>(&other)) {
            return static_cast<const DomainImpl&>(*this) <= *other_impl;
        }
        return false;
    }
};

}

#endif