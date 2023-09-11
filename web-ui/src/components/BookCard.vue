<script lang="ts">
import { computed } from 'vue';
import { useUserStore } from '@/stores/user';

interface Book {
    id: number;
    title: string;
    primary_cover: string | null;
}


export default {
    name: "BookCard",
    props: {
        book: {
            type: Object as () => Book,
            required: true
        }
    },

    computed: {
        bookCover() {
            if (this.book.primary_cover) {
                return `/api/v1/images/covers/${this.book.primary_cover}`;
            } else {
                return '/api/v1/images/covers/placeholder';
            }
        }
    },
    // mounted() {
    //     // A neat trick to get the image from the server when using Auth headers
    //     fetch(`/api/v1/images/covers/${this.book.id}`, {
    //         headers: {
    //             'Authorization': 'Bearer ' + useUserStore().user.bearerToken,
    //         }
    //     })
    //         .then(response => response.blob())
    //         .then(data => {

    //             const div_el = this.$refs.cardImg as HTMLDivElement | undefined;

    //             if (div_el) {
    //                 let url = URL.createObjectURL(data);
    //                 div_el.style.backgroundImage = `url(${url})`;
    //             }
    //         })
    //         .catch(error => {
    //             console.error('Error fetching book data:', error);
    //         });


    // }
}
</script>

<template>
    <div class="card" @click="$router.push(`/book/${book.id}`)">
        <div class="card-top">
            <div class="card-img" ref="cardImg" :style="{ backgroundImage: `url(${bookCover})` }">

            </div>
        </div>

        <div class="card-text">
            <span>{{ book.title }}</span>
        </div>
    </div>
</template>

<style scoped>
.card {
    background-color: black;
    width: 180px;
    max-width: 200px;
    height: 270px;
    box-sizing: border-box;
    position: relative;
    border-radius: 5px;
    cursor: pointer;

}

.card-top {
    width: 100%;
    height: 85%;
    position: relative;
}

.card-img {
    width: 100%;
    height: 100%;
    background-color: grey;
    background-size: cover;
    border-radius: 5px;
}

.card-text {
    position: relative;
    text-overflow: ellipsis;
    overflow: hidden;
    white-space: nowrap;
}
</style>