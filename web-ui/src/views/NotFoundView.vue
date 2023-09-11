<script lang="ts">
import TheNavigationContainer from '@/components/TheNavigationContainer.vue';
export default {
    name: "NotFoundView",
    components: {
        TheNavigationContainer
    }
}

</script>

<template>
    <TheNavigationContainer>
        <div class="not-found-container">
            <div class="column">
                <h1>404</h1>
                <h2>Page not found</h2>
                <p>Sorry, the page you are looking for does not exist.</p>
                <button @click="$router.push('/')">Go to home</button>
            </div>
        </div>
    </TheNavigationContainer>
</template>

<style scoped>
.not-found-container {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;

}

.column {
    margin: 100px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 20px;
    height: 100%;

}
</style>