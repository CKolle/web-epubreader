<script lang="ts">
import TheNavigationContainer from '@/components/TheNavigationContainer.vue';
import { useBookStore } from '@/stores/book';
interface Book {
    id: number;
    title: string;
    primary_cover: string | null;
}

export default {
    name: "BookView",
    components: {
        TheNavigationContainer
    },
    data() {
        return {
            book_id: this.$route.params.id as string,
            books: useBookStore().books,
        }
    },
    computed: {
        book() {
            return this.books.find(book => book.id === +this.book_id);
        },
        bookCover() {
            if (this.book?.primary_cover) {
                return `/api/v1/images/covers/${this.book.primary_cover}`;
            } else {
                return '/api/v1/images/covers/placeholder';
            }
        }
    },

    mounted() {
        // A neat trick to get the image from the server when using Auth headers

        if (!this.book) {
            useBookStore().fetchBook(+this.book_id);
        }

        // fetch(`/api/v1/images/covers/${this.book.id}`, {
        //     headers: {
        //         'Authorization': 'Bearer ' + useUserStore().user.bearerToken,
        //     }
        // })
        //     .then(response => response.blob())
        //     .then(data => {

        //         const div_el = this.$refs.img as HTMLDivElement | undefined;

        //         if (div_el) {
        //             let url = URL.createObjectURL(data);
        //             div_el.style.backgroundImage = `url(${url})`;
        //         }
        //     })
        //     .catch(error => {
        //         console.error('Error fetching book data:', error);
        //     });


    },
    // async beforeMount() {
    //     const { getBookById } = useBookStore();
    //     console.log(this.book_id);
    //     let book = getBookById(+this.book_id);
    //     console.log(book);

    //     // For chaching purposes, we try to get the book from the store first
    //     if (!book) {
    //         // Try to fetch book from API
    //         await useBookStore().fetchBook(+this.book_id)
    //         book = getBookById(+this.book_id);
    //     }

    //     // If book is still null, redirect to 404
    //     if (!book) {
    //         this.$router.push("/404");
    //     } else {
    //         this.book = book;
    //     }
    // },
}
</script>

<template>
    <TheNavigationContainer>
        <div class="top-container" v-if="book">
            <div class="img-container">
                <div class="img" ref="img" :style="{ backgroundImage: `url(${bookCover})` }">

                </div>
            </div>
            <div class="info-container">
                <div class="row">
                    <h1>{{ book.title }}</h1>
                </div>
                <div class="row-last">
                    <button @click="$router.push(`/read/${book.id}`)">Read</button>
                    <button>Download</button>
                </div>


            </div>

        </div>
    </TheNavigationContainer>
</template>

<style scoped>
h1 {
    margin: 0;
}

.info-container {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    margin-left: 20px;
    padding: 20px;
    box-sizing: border-box;
}

.img {
    width: 100%;
    height: 100%;
    background-size: cover;
    background-position: center;
    border-radius: 5px;
}



.top-container {
    display: flex;
    flex-direction: row;
    padding: 20px;
    margin-top: 20px;
    width: 100%;
    flex-wrap: wrap;

}

.img-container {
    width: 200px;
    min-width: 200px;
    height: 300px;
    background-color: #333;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
}
</style>