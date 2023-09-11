<script lang="ts">
import TheNavigationContainer from '@/components/TheNavigationContainer.vue';
export default {
    name: "SettingsView",
    components: {
        TheNavigationContainer
    }
}

</script>

<template>
    <TheNavigationContainer>
        <div class="settings-container">
            <div class="column">
                <div class="card">
                    <h2>Server Info</h2>
                    <table>
                        <tr>
                            <td>Version</td>
                            <td>0.1.0</td>
                        </tr>
                        <tr>
                            <td>Server Name</td>
                            <td>Server Name</td>
                        </tr>
                        <tr>
                            <td>Server Name</td>
                            <td>Server Name</td>
                        </tr>
                        <tr>
                            <td>Server Name</td>
                            <td>Server Name</td>
                        </tr>
                    </table>
                </div>

            </div>
            <div class="column">
                <div class="card">

                    <div class="card-list-item" @click="$router.push('/settings/account')">
                        <span>Account Settings</span>
                    </div>
                    <div class="card-list-item" @click="$router.push('/settings/libraries')">
                        <span>Libraries</span>
                    </div>
                </div>



            </div>
        </div>
    </TheNavigationContainer>
</template>

<style scoped>
.settings-container {
    padding: 1rem;
    display: flex;
    flex-direction: row;
    justify-content: center;
    gap: 1rem;
}

.column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 40%;
    max-width: 600px;
}

.card {
    background-color: var(--main-colour-dark);
    border-radius: 5px;
    border: 1px solid #000;
}

.card-list-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: 0 1rem;
    height: 65px;
    box-sizing: border-box;
}

.card-list-item:hover {
    background-color: var(--main-colour-light);
    cursor: pointer;
}

.card-list-item:first-of-type {
    border-top-left-radius: 5px;
    border-top-right-radius: 5px;
}

.card-list-item:last-of-type {
    border-bottom-left-radius: 5px;
    border-bottom-right-radius: 5px;
}

@media only screen and (max-width: 1000px) {
    .settings-container {
        flex-direction: column;
        align-items: center;
    }

    .column {
        width: 100%;
    }
}
</style>
