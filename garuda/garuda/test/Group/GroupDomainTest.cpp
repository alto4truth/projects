#include <gtest/gtest.h>
#include "garuda/Group/GroupDomain.h"
#include "garuda/Group/GroupDomainManager.h"
#include "garuda/Group/GroupDomainBuilder.h"

using namespace garuda;

TEST(GroupDomainTest, Basic) {
    GroupDomain group("test");
    EXPECT_EQ(group.getName(), "test");
    EXPECT_FALSE(group.isTop());
    EXPECT_FALSE(group.isBottom());
}

TEST(GroupDomainTest, Members) {
    GroupDomain group("test");

    auto int_domain = std::make_unique<IntegerDomain>(0, 100);
    group.addMember("x", std::move(int_domain));

    EXPECT_TRUE(group.hasMember("x"));
    EXPECT_EQ(group.getMemberCount(), 1);

    auto member = group.getMember("x");
    EXPECT_NE(member, nullptr);
}

TEST(GroupDomainTest, RemoveMember) {
    GroupDomain group("test");
    group.addMember("x", std::make_unique<IntegerDomain>(0, 100));

    EXPECT_TRUE(group.hasMember("x"));
    group.removeMember("x");
    EXPECT_FALSE(group.hasMember("x"));
}

TEST(GroupDomainTest, Dependencies) {
    GroupDomain group("test");
    group.addDependency("a", "b");

    EXPECT_TRUE(group.hasDependency("a", "b"));
    EXPECT_FALSE(group.hasDependency("b", "a"));
}

TEST(GroupDomainTest, Clone) {
    GroupDomain original("test");
    original.addMember("x", std::make_unique<IntegerDomain>(0, 100));

    auto cloned = original.clone();
    EXPECT_EQ(cloned->getName(), "test");
    EXPECT_TRUE(cloned->hasMember("x"));
}

TEST(GroupDomainTest, Join) {
    GroupDomain a("a");
    a.addMember("x", std::make_unique<IntegerDomain>(0, 10));

    GroupDomain b("b");
    b.addMember("y", std::make_unique<IntegerDomain>(5, 20));

    auto joined = a.join(b);
    EXPECT_TRUE(joined->hasMember("x") || joined->hasMember("y"));
}

TEST(GroupDomainManagerTest, CreateGroup) {
    GroupDomainManager manager;

    auto group = manager.createGroup("test");
    EXPECT_NE(group, nullptr);
    EXPECT_TRUE(manager.hasGroup("test"));
}

TEST(GroupDomainManagerTest, GetGroup) {
    GroupDomainManager manager;
    manager.createGroup("test");

    auto group = manager.getGroup("test");
    EXPECT_NE(group, nullptr);
    EXPECT_EQ(group->getName(), "test");
}

TEST(GroupDomainManagerTest, RemoveGroup) {
    GroupDomainManager manager;
    manager.createGroup("test");
    EXPECT_TRUE(manager.hasGroup("test"));

    manager.removeGroup("test");
    EXPECT_FALSE(manager.hasGroup("test"));
}

TEST(GroupDomainManagerTest, GetGroupNames) {
    GroupDomainManager manager;
    manager.createGroup("a");
    manager.createGroup("b");

    auto names = manager.getGroupNames();
    EXPECT_EQ(names.size(), 2);
}

TEST(GroupDomainBuilderTest, Basic) {
    auto group = GroupDomainBuilder::create("test").build();
    EXPECT_EQ(group->getName(), "test");
}

TEST(GroupDomainBuilderTest, WithMembers) {
    auto group = GroupDomainBuilder::create("test")
        .addMember("x", std::make_unique<IntegerDomain>(0, 100))
        .addMember("y", std::make_unique<BooleanDomain>(true))
        .build();

    EXPECT_EQ(group->getMemberCount(), 2);
}

TEST(GroupDomainBuilderTest, WithConfig) {
    GroupDomainConfig config;
    config.max_iterations = 50;

    auto group = GroupDomainBuilder::create("test")
        .setConfig(config)
        .build();

    EXPECT_EQ(group->getConfig().max_iterations, 50);
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}