import { defineStore } from "pinia";
import { fetchWrapper } from "@/utils/requestHelper";

interface Book {
    id: number;
    title: string;
    book_asset: string;
    primary_cover: string | null;
}

export const useBookStore = defineStore("book", {
    state: () => {
        return {
            loading: false,
            books: [] as Book[],
        }
    },
    actions: {
        async fetchBooks() {
            this.loading = true;
            const result = await fetchWrapper.get("/api/v1/book") as Book[] | null;
            this.loading = false;
            if (!result) {
                return;
            }
            this.books = result;
            return result;

        },
        async fetchBook(id: number) {
            this.loading = true;
            const result = await fetchWrapper.get(`/api/v1/book/${id}`) as Book | null;
            this.loading = false;
            if (!result) {
                return;
            }
            // Needs to check if the book is already in the list

            const index = this.books.findIndex(book => book.id === id);

            if (index !== -1) {
                this.books[index] = result;
            }

            this.books.push(result);

            return result;
        }
    },
    getters: {
        getBookById(state) {
            return (id: number) => {
                return state.books.find((book) => book.id === id);
            }
        }
    },
});