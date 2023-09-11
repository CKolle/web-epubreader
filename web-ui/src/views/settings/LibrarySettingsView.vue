<script lang="ts">
import LibraryCard from '@/components/settings/LibraryCard.vue';
import { useLibraryStore } from '@/stores/library';
import TheNavigationContainer from '@/components/TheNavigationContainer.vue';
import { ref } from 'vue';

export default {
    name: "LibrarySettingsView",
    data() {
        return {
            name: "",
            path: ""
        }
    },
    components: {
        LibraryCard,
        TheNavigationContainer
    },
    methods: {
        async addLibrary() {
            await useLibraryStore().addLibrary(this.path, this.name);
            this.name = "";
            this.path = "";


            const dialog_el = this.$refs.dialog as HTMLDialogElement | undefined;
            dialog_el?.close();
        }

    },
    computed: {
        libraries() {
            return useLibraryStore().libraries;
        },
        isLoading() {
            return useLibraryStore().loading;
        }
    },
    beforeMount() {
        useLibraryStore().fetchLibraries();
    }
}

</script>

<template>
    <TheNavigationContainer>
        <div class="library-settings-container">
            <div class="top-bar">
                <h1>Libraries</h1>
                <div class="top-buttons">
                    <button @click="$router.push('/settings')">Back to settings</button>
                    <button class="btn btn-primary" onclick="d.showModal()">Add Library</button>
                </div>
            </div>

            <div class="card" v-if="!isLoading">
                <LibraryCard v-for="library in libraries" :key="library.id" :library="library" />
            </div>
        </div>
        <dialog id="d" ref="dialog" eager>
            <div class="top-bar">
                <h3>Add Library</h3>
                <button onclick="d.close()">Close</button>
            </div>
            <div class="form-container">
                <form @submit.prevent="addLibrary">
                    <label for="name">Name</label>
                    <input type="text" id="name" name="name" v-model="name" required minlength="1" />
                    <label for="path">Path</label>
                    <input type="text" id="path" name="path" v-model="path" required minlength="1" />

                    <button type="submit">Add Library</button>
                </form>
            </div>

        </dialog>
    </TheNavigationContainer>
</template>

<style scoped>
dialog {
    width: 700px;
    background-color: var(--main-colour-dark);
    border-radius: 5px;
    border: 1px solid #000;
    color: var(--font-colour);
    padding: 0;
}

input {
    width: 100%;
    padding: 8px;
    display: inline-block;
    box-sizing: border-box;
    margin-bottom: 30px;
    color: var(--font-colour);
    background-color: var(--main-colour-light);
    border: none;
    border-radius: 4px;
}

.form-container {
    padding: 20px;
}

.top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #000;
    padding: 10px 20px;
}

.library-settings-container {
    display: flex;
    flex-direction: column;
    padding: 0 20px;

}

.top-buttons {
    gap: 10px;
    display: flex;
    flex-wrap: wrap;
}

.top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
}

.card {
    background-color: var(--main-colour-dark);
    padding: 20px;
    border-radius: 5px;
    border: 1px solid #000;
    margin-top: 20px;

}

.delete-btn {
    background-color: var(--main-colour-red);
}

.delete-btn:hover {
    background-color: var(--main-colour-red-dark);
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
    gap: 10px;
}
</style>