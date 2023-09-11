<script lang="ts">

import { useToastStore } from '@/stores/toast';
export default {
    name: "TheToaster",
    computed: {
        toast() {
            return useToastStore().toast
        },
    },
    methods: {
        closeToast() {
            console.log("closeToast");
            useToastStore().closeToast();
        },
    },
}

</script>

<template>
    <div class="toast" :class="toast.type" v-if="toast">
        <div class="container">
            <span>
                <!-- Todo: Add some status icons -->
            </span>
            <p>
                {{ toast.message }}
            </p>
            <span class="close" @click="closeToast">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-x-circle"
                    viewBox="0 0 16 16">
                    <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
                    <path
                        d="M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708z" />
                </svg>
            </span>

        </div>
    </div>
</template>

<style scoped>
.toast {
    z-index: 1000;
}

span {
    display: flex;
    justify-content: center;
}

p {
    margin: 10px;
}


@keyframes slideIn {
    0% {
        top: -50px;
    }
}

.toast {
    position: fixed;
    left: 50%;
    top: 40px;
    transform: translateX(-50%);
    background-color: #333;
    padding: 5px 10px;
    border-radius: 5px;
    border: 1px solid #000;
    animation-name: slideIn;
    animation-duration: 0.5s;
    animation-fill-mode: forwards;
}

.container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-direction: row;
}

.close {
    cursor: pointer;
}

.error {
    background-color: rgb(167, 2, 2);
}

.success {
    background-color: rgb(2, 167, 2);
}
</style>