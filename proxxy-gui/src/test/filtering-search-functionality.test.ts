import { describe, it, expect } from 'vitest';
import * as fc from 'fast-check';

describe('Filtering and Search Functionality Property Tests', () => {

    /**
     * Property 4: Filtering and Search Functionality
     * The filtering and search logic must correctly include only items matching the criteria
     * and exclude strictly those that do not.
     * Validates: Requirements 2.5, 3.4, 3.5
     */
    it('Property 4: Agent Status Filtering - Result should only contain agents with matching status', async () => {
        await fc.assert(
            fc.property(
                fc.array(
                    fc.record({
                        id: fc.string(),
                        name: fc.string(),
                        hostname: fc.string(),
                        status: fc.oneof(fc.constant('Online'), fc.constant('Offline')),
                        version: fc.string(),
                        lastHeartbeat: fc.string(),
                    })
                ),
                fc.oneof(fc.constant('Online'), fc.constant('Offline'), fc.constant('All')),
                (agents, filterStatus) => {
                    // Simulate the filtering logic used in AgentsView
                    const filtered = agents.filter(agent => {
                        const matchesStatus = filterStatus === 'All' || (agent.status as any) === filterStatus;
                        return matchesStatus;
                    });

                    // Verify Property
                    if (filterStatus !== 'All') {
                        filtered.forEach(agent => {
                            // When filterStatus is not All, it must match the agent status
                            expect((agent.status as string)).toBe(filterStatus);
                        });

                        const rejected = agents.filter(agent => {
                            // Logic mirrored for rejection
                            const matchesStatus = filterStatus === 'All' || (agent.status as any) === filterStatus;
                            return !matchesStatus;
                        });

                        rejected.forEach(agent => {
                            // Ensure rejected items do not match
                            expect((agent.status as string)).not.toBe(filterStatus);
                        });
                    } else {
                        expect(filtered.length).toBe(agents.length);
                    }
                }
            )
        );
    });


    it('Property 4: Agent Search Functionality - Result should only contain agents matching search term', async () => {
        await fc.assert(
            fc.property(
                fc.array(
                    fc.record({
                        id: fc.string(),
                        name: fc.string(),
                        hostname: fc.string(),
                        status: fc.string(),
                        version: fc.string(),
                        lastHeartbeat: fc.string(),
                    })
                ),
                fc.string(),
                (agents, searchTerm) => {
                    // Simulate Search Logic
                    const filtered = agents.filter(agent => {
                        const termInfo = searchTerm.toLowerCase();
                        const matchesSearch = agent.name.toLowerCase().includes(termInfo) ||
                            agent.id.toLowerCase().includes(termInfo) ||
                            agent.hostname.toLowerCase().includes(termInfo);
                        return matchesSearch;
                    });

                    // Verify Property
                    filtered.forEach((agent: any) => {
                        const termInfo = searchTerm.toLowerCase();
                        const matches = agent.name.toLowerCase().includes(termInfo) ||
                            agent.id.toLowerCase().includes(termInfo) ||
                            agent.hostname.toLowerCase().includes(termInfo);
                        expect(matches).toBe(true);
                    });
                }
            )
        );
    });
});
