<script lang="ts">
interface Library {
    id: string;
    name: string;
    path: string;
}

import { useLibraryStore } from '@/stores/library';

export default {
    name: "LibraryCard",
    methods: {
        async deleteLibrary() {
            await useLibraryStore().deleteLibrary(this.library.id);
        },
        async scanLibrary() {
            await useLibraryStore().scanLibrary(this.library.id);
        }
    },
    props: {
        library: {
            type: Object as () => Library,
            required: true
        }
    }
}
</script>

<template>
    <div class="library-item">
        <div class="library-item-info">
            <h2>{{ library.name }} </h2>
            <p>{{ library.path }}</p>
        </div>
        <div class="library-item-buttons">
            <button class="edit-btn">Edit</button>
            <button class="delete-btn" @click="deleteLibrary">Delete</button>
            <button class="scan-btn" @click="scanLibrary">Scan</button>
        </div>
    </div>
</template>

<style scoped>
.delete-btn {
    background-color: var(--main-colour-red);
}

.delete-btn:hover {
    background-color: var(--main-colour-red-dark);
}

.scan-btn {
    background-color: grey;
}

.scan-btn:hover {
    background-color: darkgrey;
}

.library-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
}

.library-item-buttons {
    display: flex;
    flex-direction: column;
    gap: 10px;
}
</style>