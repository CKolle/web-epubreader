import { defineStore } from "pinia";
import { fetchWrapper } from "@/utils/requestHelper";


export const useUserStore = defineStore("user", {
    state: () => {
        return {
            isLoggedIn: JSON.parse(localStorage.getItem('isLoggedIn') || 'false') as boolean,
            user: JSON.parse(localStorage.getItem('user') || '{}') as User,
        }
    },
    actions: {
        async login(username: string, password: string, rememberMe: boolean) {
            const response = await fetchWrapper.post('/api/v1/auth/login', { username, password }) as LoginResponse | null;
            if (!response) {
                return;
            }

            this.isLoggedIn = true;
            this.user = {
                username,
                bearerToken: response.bearer_token,
            };

            if (rememberMe) {
                localStorage.setItem('user', JSON.stringify(this.user));
                localStorage.setItem('isLoggedIn', 'true');
            }

        }
    }
});



interface User {
    username: string;
    bearerToken: string;
}

interface LoginResponse {
    bearer_token: string;
}