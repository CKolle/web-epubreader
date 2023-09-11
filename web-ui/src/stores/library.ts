import { defineStore } from "pinia";
import { fetchWrapper } from "@/utils/requestHelper";

export const useLibraryStore = defineStore("library", {
    state: () => {
        return {
            loading: false,
            libraries: [] as Library[],
        }
    },
    actions: {
        async fetchLibraries() {
            this.loading = true;
            const result = await fetchWrapper.get("/api/v1/library") as Library[] | null;
            this.loading = false;
            if (!result) {
                return;
            }
            this.libraries = result;
        },
        async addLibrary(path: string, name: string) {
            const result = await fetchWrapper.post("/api/v1/library", { path, name }) as Library | null;
            if (!result) {
                return;
            }
            this.libraries.push(result);
        },

        async deleteLibrary(id: string) {
            const result = await fetchWrapper.delete(`/api/v1/library/${id}`) as boolean | null;
            if (!result) {
                return;
            }
            this.libraries = this.libraries.filter(library => library.id !== id);
        },

        async scanLibrary(library_id: string) {
            await fetchWrapper.post("/api/v1/library/scan", { library_id }) as boolean | null;
        },
    }
});

interface Library {
    id: string;
    name: string;
    path: string;
}