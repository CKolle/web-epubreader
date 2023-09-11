<script lang="ts">
import BookCard from '@/components/BookCard.vue';
import TheNavigationContainer from '@/components/TheNavigationContainer.vue';
import { useBookStore } from '@/stores/book';

export default {
    name: "TotalCollectionView",

    computed: {
        books() {
            return useBookStore().books;
        },
        isLoading() {
            return useBookStore().loading;
        }
    },
    components: {
        BookCard,
        TheNavigationContainer
    },
    beforeMount() {
        useBookStore().fetchBooks();
    },
}

</script>

<template>
    <TheNavigationContainer>
        <div class="container">
            <h1>Total Collections</h1>
            <div class="center-container" v-if="!isLoading">
                <div class="item-container">
                    <BookCard v-for="book in books" :key="book.id" :book="book" />
                </div>
            </div>
        </div>
    </TheNavigationContainer>
</template>
  
<style scoped>
.container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 1rem;
    position: relative;
}

.center-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
}

.item-container {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 1rem;
    width: 100%;
}

@media (max-width: 600px) {
    .item-container {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        flex-shrink: 2;
        flex-basis: 50px;

    }
}
</style>
  