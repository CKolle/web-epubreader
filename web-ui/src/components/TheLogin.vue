<script lang="ts">
import { useUserStore } from '@/stores/user';
export default {
    name: "TheLogin",
    data() {
        return {
            username: "",
            password: "",
            rememberMe: false,
        }
    },
    methods: {
        async login() {
            await useUserStore().login(this.username, this.password, this.rememberMe);
            if (useUserStore().isLoggedIn) {
                const redirect = this.$route.query.redirect || "/";
                this.$router.push(redirect as string);
            }
        },
    },
}
</script>

<template>
    <div class="center-container">
        <div class="column-container">
            <div class="logo-container">

                <h1>Login</h1>
            </div>
            <div class="login-form-container">
                <form @submit.prevent="login">
                    <label for="username"><b>Username</b></label>
                    <input type="text" placeholder="Username" id="username" v-model="username" required />

                    <label for="password"><b>Password</b></label>
                    <input type="password" placeholder="Password" id="password" v-model="password" required />

                    <label for="remember-me"><b>Remember me</b></label>
                    <input type="checkbox" id="remember-me" v-model="rememberMe" />

                    <button type="submit">Login</button>
                </form>
            </div>
        </div>
    </div>
</template>

<style scoped>
.center-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    background-color: var(--main-colour-light);
    background-size: cover;
    background-repeat: no-repeat;
    background-position: center;
}

.column-container {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    width: 400px;
    padding: 16px;
    background-color: var(--background-colour-trans);
    border-radius: 4px;
    border: 1px solid #000;
    gap: 16px;
}

input[type="text"],
input[type="password"] {
    width: 100%;
    padding: 8px;
    display: inline-block;
    box-sizing: border-box;
    margin-bottom: 30px;
    color: var(--font-colour);
    background-color: var(--main-colour-dark);
    border: none;
    border-radius: 4px;
}

input[type="checkbox"] {
    cursor: pointer;
}


button {
    width: 100%;
    margin-top: 15px;
    padding: 4px;
    cursor: pointer;
    color: var(--main-colour);
    background-color: var(--secondary-colour);
    border: none;
    border-radius: 4px;
}

button:hover {
    background-color: var(--secondary-colour-dark);
}
</style>