#include <iostream>
#include "garuda/Domain/AbstractDomain.h"
#include "garuda/Domain/IntegerDomain.h"
#include "garuda/Domain/BooleanDomain.h"
#include "garuda/Group/GroupDomain.h"
#include "garuda/Group/GroupDomainManager.h"
#include "garuda/Update/ProgressUpdate.h"

using namespace garuda;

int main() {
    std::cout << "Garuda Static Analysis Framework\n";
    std::cout << "================================\n\n";

    auto group_mgr = std::make_shared<GroupDomainManager>();
    
    auto g1 = group_mgr->createGroup("values");
    g1->setTop();
    std::cout << "Created group: " << g1->getName() << "\n";
    std::cout << "  isTop: " << (g1->isTop() ? "yes" : "no") << "\n";
    
    auto g2 = group_mgr->createGroup("pointers");
    g2->addMember("ptr1", std::make_unique<IntegerDomain>(0, 100));
    std::cout << "Created group: " << g2->getName() << "\n";
    std::cout << "  members: " << g2->getMemberCount() << "\n";

    auto g3 = group_mgr->createGroup("booleans");
    g3->addMember("flag", std::make_unique<BooleanDomain>(true));
    std::cout << "Created group: " << g3->getName() << "\n";
    std::cout << "  members: " << g3->getMemberCount() << "\n";
    
    std::cout << "\nGroups: " << group_mgr->getGroupCount() << "\n";
    for (const auto& name : group_mgr->getGroupNames()) {
        std::cout << "  - " << name << "\n";
    }
    
    std::cout << "\nCycles: " << (group_mgr->detectCycles() ? "yes" : "no") << "\n";
    std::cout << "Topo: ";
    for (const auto& n : group_mgr->topologicalSort()) {
        std::cout << n << " ";
    }
    std::cout << "\n";

    ProgressUpdate progress("Analysis");
    progress.setLevel(ProgressLevel::DETAILED);
    progress.setStatus(UpdateStatus::IN_PROGRESS);
    progress.recordIteration(1);
    progress.recordProgress(0.5);
    std::cout << "\nProgress: " << progress.getProgress() * 100 << "%\n";

    std::cout << "\nDone!\n";
    
    return 0;
}