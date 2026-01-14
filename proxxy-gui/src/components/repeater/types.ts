import { RepeaterTask as StoreTask } from "@/store/repeaterStore";

export interface RepeaterAgent {
    id: string;
    name: string;
    hostname: string;
    status: string;
    version: string;
    type: 'local' | 'cloud';
}

export type RepeaterTask = StoreTask;
