import { useUserStore } from '../stores/user';
import { useToastStore, ToastType } from '@/stores/toast';



enum Method {
    GET = 'GET',
    POST = 'POST',
    PUT = 'PUT',
    DELETE = 'DELETE',
}

interface Headers {
    [key: string]: string;
}

interface RequestOptions {
    method: Method;
    headers: Headers;
    body?: string;
}


function makeRequest(method: Method) {
    async function send_request(url: string, body: object | null = null) {

        const headers: Headers = {};
        if (useUserStore().isLoggedIn) {
            headers['Authorization'] = `Bearer ${useUserStore().user?.bearerToken}`;
        }

        const options: RequestOptions = {
            method,
            headers,
        };

        if (body) {
            options['body'] = JSON.stringify(body);
            options.headers['Content-Type'] = 'application/json';
        }

        return await fetch(url, options).then(handleResponse);

    }

    return send_request;
}


async function handleResponse(response: Response) {
    if (!response.ok) {
        try {
            const json = await response.json();
            if (json) {
                useToastStore().showToast(json.error, ToastType.ERROR);
            }

            return;
        } catch (error) {
            // Ment for error that are not meant to be shown to the user
            return;
        }
    }

    try {
        const json_data = await response.json();
        if (json_data.success) {
            useToastStore().showToast(json_data.success, ToastType.SUCCESS);
            return true;
        }
        return json_data;

    } catch (error) {
        // Ment for error that are not meant to be shown to the user
        return;
    }
}


export const fetchWrapper = {
    get: makeRequest(Method.GET),
    post: makeRequest(Method.POST),
    put: makeRequest(Method.PUT),
    delete: makeRequest(Method.DELETE),
}