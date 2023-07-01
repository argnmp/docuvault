import { configureStore } from "@reduxjs/toolkit";
import { debounce } from "debounce";

import commonReducer from "./common";
import authReducer from "./auth";
import resourceReducer from './resource';

const KEY = "redux";
export function loadState() {
    try {
        const serializedState = localStorage.getItem(KEY);
        if (!serializedState) return undefined;
        return JSON.parse(serializedState);
    } catch (e) {
        return undefined;
    }
}

export async function saveState(state) {
    try {
        const serializedState = JSON.stringify(state);
        localStorage.setItem(KEY, serializedState);
    } catch (e) {
        // Ignore
    }
}

export const store = configureStore({
    reducer: {
        auth: authReducer,
        common: commonReducer,
        resource: resourceReducer,
    },
    preloadedState: loadState(),
});
store.subscribe(
    debounce(() => {
        saveState(store.getState());
    }, 800)
);
