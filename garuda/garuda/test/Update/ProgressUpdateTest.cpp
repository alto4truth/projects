#include <gtest/gtest.h>
#include "garuda/Update/ProgressUpdate.h"
#include "garuda/Update/ProgressUpdateTracker.h"

using namespace garuda;

TEST(ProgressUpdateTest, Basic) {
    ProgressUpdate update("test");
    EXPECT_EQ(update.getName(), "test");
    EXPECT_EQ(update.getIteration(), 0u);
    EXPECT_EQ(update.getProgress(), 0.0);
}

TEST(ProgressUpdateTest, RecordIteration) {
    ProgressUpdate update("test");
    update.recordIteration(5);
    EXPECT_EQ(update.getIteration(), 5u);
}

TEST(ProgressUpdateTest, RecordProgress) {
    ProgressUpdate update("test");
    update.recordProgress(0.5);
    EXPECT_EQ(update.getProgress(), 0.5);

    update.recordProgress(1.5);
    EXPECT_EQ(update.getProgress(), 1.0);

    update.recordProgress(-0.5);
    EXPECT_EQ(update.getProgress(), 0.0);
}

TEST(ProgressUpdateTest, Deltas) {
    ProgressUpdate update("test");

    ProgressDelta delta1;
    delta1.key = "x";
    delta1.old_value = "0";
    delta1.new_value = "10";
    delta1.improved = true;
    update.addDelta(delta1);

    EXPECT_TRUE(update.hasImproved());
    EXPECT_EQ(update.getDeltas().size(), 1);
}

TEST(ProgressUpdateTest, Snapshots) {
    ProgressUpdate update("test");

    ProgressSnapshot snapshot;
    snapshot.iteration = 1;
    snapshot.progress_ratio = 0.5;
    update.addSnapshot(snapshot);

    EXPECT_EQ(update.getSnapshots().size(), 1);
}

TEST(ProgressUpdateTest, Reset) {
    ProgressUpdate update("test");
    update.recordIteration(5);
    update.recordProgress(0.5);

    update.reset();
    EXPECT_EQ(update.getIteration(), 0u);
    EXPECT_EQ(update.getProgress(), 0.0);
}

TEST(ProgressUpdateTest, ClearHistory) {
    ProgressUpdate update("test");
    update.addSnapshot(ProgressSnapshot());
    update.addSnapshot(ProgressSnapshot());

    update.clearHistory();
    EXPECT_EQ(update.getSnapshots().size(), 0);
}

TEST(ProgressUpdateTest, MarkStable) {
    ProgressUpdate update("test");
    update.markStable();
    EXPECT_TRUE(update.isStable());
}

TEST(ProgressUpdateTrackerTest, Basic) {
    ProgressUpdateTracker tracker("test");
    EXPECT_EQ(tracker.getCurrentIteration(), 0u);
}

TEST(ProgressUpdateTrackerTest, Initialize) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);

    EXPECT_EQ(tracker.getTotalIterations(), 100u);
}

TEST(ProgressUpdateTrackerTest, RecordIteration) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);
    tracker.recordIteration(50, 0.5);

    EXPECT_EQ(tracker.getCurrentIteration(), 50u);
    EXPECT_EQ(tracker.getProgress(), 0.5);
}

TEST(ProgressUpdateTrackerTest, HasConverged) {
    ProgressUpdateTracker tracker("test");
    tracker.setMaxStagnation(3);
    tracker.setConvergenceThreshold(0.001);

    tracker.initialize(100);
    tracker.recordIteration(1, 0.1);
    tracker.recordIteration(2, 0.05);
    tracker.recordIteration(3, 0.001);

    EXPECT_FALSE(tracker.hasConverged());
}

TEST(ProgressUpdateTrackerTest, ShouldTerminate) {
    ProgressUpdateTracker tracker("test");

    tracker.initialize(10);
    EXPECT_FALSE(tracker.shouldTerminate());

    tracker.recordIteration(10, 1.0);
    EXPECT_TRUE(tracker.shouldTerminate());
}

TEST(ProgressUpdateTrackerTest, ComputeProgressRate) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);

    tracker.recordIteration(1, 0.1);
    tracker.recordIteration(2, 0.2);
    tracker.recordIteration(3, 0.3);

    double rate = tracker.computeProgressRate();
    EXPECT_GE(rate, 0.0);
}

TEST(ProgressUpdateTrackerTest, Reset) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);
    tracker.recordIteration(50, 0.5);

    tracker.reset();
    EXPECT_EQ(tracker.getCurrentIteration(), 0u);
    EXPECT_EQ(tracker.getTotalIterations(), 0u);
}

TEST(ProgressUpdateTrackerTest, Checkpoints) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);
    tracker.setCheckpointInterval(5);

    tracker.recordIteration(1, 0.1);
    tracker.recordIteration(2, 0.2);
    tracker.recordIteration(3, 0.3);
    tracker.recordIteration(4, 0.4);
    tracker.recordIteration(5, 0.5);

    EXPECT_GE(tracker.getCheckpointCount(), 0u);
}

TEST(ProgressUpdateTrackerTest, GetElapsedTime) {
    ProgressUpdateTracker tracker("test");
    tracker.initialize(100);
    tracker.recordIteration(10, 0.1);

    double elapsed = tracker.getElapsedTime();
    EXPECT_GE(elapsed, 0.0);
}

int main(int argc, char** argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}