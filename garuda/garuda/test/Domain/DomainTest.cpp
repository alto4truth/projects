#include <gtest/gtest.h>
#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/IntegerDomain.h"
#include "garuda/Domain/BooleanDomain.h"

using namespace garuda;

TEST(IntegerDomainTest, TopAndBottom) {
    auto top = std::make_unique<IntegerDomain>();
    EXPECT_TRUE(top->isTop());
    EXPECT_FALSE(top->isBottom());

    auto bottom = std::make_unique<IntegerDomain>();
    bottom->setBottom();
    EXPECT_FALSE(bottom->isTop());
    EXPECT_TRUE(bottom->isBottom());
}

TEST(IntegerDomainTest, Constructors) {
    IntegerDomain single(42);
    EXPECT_EQ(single.getLowerBound(), 42);
    EXPECT_EQ(single.getUpperBound(), 42);
    EXPECT_TRUE(single.isSingleton());

    IntegerDomain interval(0, 100);
    EXPECT_EQ(interval.getLowerBound(), 0);
    EXPECT_EQ(interval.getUpperBound(), 100);
    EXPECT_FALSE(interval.isSingleton());
}

TEST(IntegerDomainTest, Join) {
    IntegerDomain a(0, 10);
    IntegerDomain b(5, 20);

    auto joined = a.join(b);
    EXPECT_EQ(joined->getLowerBound(), 0);
    EXPECT_EQ(joined->getUpperBound(), 20);
}

TEST(IntegerDomainTest, Meet) {
    IntegerDomain a(0, 10);
    IntegerDomain b(5, 20);

    auto met = a.meet(b);
    EXPECT_EQ(met->getLowerBound(), 5);
    EXPECT_EQ(met->getUpperBound(), 10);
}

TEST(IntegerDomainTest, MeetDisjoint) {
    IntegerDomain a(0, 5);
    IntegerDomain b(10, 20);

    auto met = a.meet(b);
    EXPECT_TRUE(met->isBottom());
}

TEST(IntegerDomainTest, Widen) {
    IntegerDomain a(0, 10);
    IntegerDomain b(5, 20);

    auto widened = a.widen(b);
    EXPECT_TRUE(widened->isTop() || widened->getLowerBound() <= 0);
}

TEST(IntegerDomainTest, Arithmetic) {
    IntegerDomain a(1, 5);
    IntegerDomain b(2, 3);

    auto sum = a.applyBinaryOp(DomainOperator::ADD, b);
    EXPECT_EQ(sum->getLowerBound(), 3);
    EXPECT_EQ(sum->getUpperBound(), 8);

    auto diff = a.applyBinaryOp(DomainOperator::SUB, b);
    EXPECT_LE(diff->getLowerBound(), 0);
}

TEST(IntegerDomainTest, Compare) {
    IntegerDomain a(5, 10);
    IntegerDomain b(0, 3);

    auto cmp = a.applyCompare(DomainOperator::COMPARE_GT, b);
    EXPECT_TRUE(cmp->getAbstractBoolean());
}

TEST(BooleanDomainTest, Basic) {
    BooleanDomain b(true);
    EXPECT_TRUE(b.getValue());
    EXPECT_FALSE(b.isTop());
    EXPECT_FALSE(b.isBottom());
}

TEST(BooleanDomainTest, Operations) {
    BooleanDomain a(true);
    BooleanDomain b(false);

    auto and_result = a.applyBinaryOp(DomainOperator::AND, b);
    EXPECT_FALSE(and_result->getAbstractBoolean());

    auto or_result = a.applyBinaryOp(DomainOperator::OR, b);
    EXPECT_TRUE(or_result->getAbstractBoolean());

    auto not_result = a.applyUnaryOp(DomainOperator::NOT);
    EXPECT_FALSE(not_result->getAbstractBoolean());
}

TEST(BooleanDomainTest, Comparison) {
    BooleanDomain a(true);
    BooleanDomain b(true);

    auto eq = a.applyCompare(DomainOperator::COMPARE_EQ, b);
    EXPECT_TRUE(eq->getAbstractBoolean());

    auto ne = a.applyCompare(DomainOperator::COMPARE_NE, b);
    EXPECT_FALSE(ne->getAbstractBoolean());
}

TEST(DomainFactory, CreateDomains) {
    auto int_domain = DomainFactory::createDomain(DomainType::INTEGER);
    EXPECT_TRUE(int_domain->isTop());

    auto const_domain = DomainFactory::createConstantIntegerDomain(42);
    EXPECT_FALSE(const_domain->isTop());

    auto interval = DomainFactory::createIntervalDomain(0, 100);
    EXPECT_FALSE(interval->isTop());
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}