import { createSlice } from "@reduxjs/toolkit";

const initialState = {
    isLogined: false,
    access_token: "",
};

export const authSlice = createSlice({
    name: 'auth',
    initialState,
    reducers: {
        login: (state, action) => {
            state.isLogined = true;
            state.access_token = action.payload.access_token; 
        },
        logout: (state) => {
            state.isLogined = false;
            state.access_token = "";
        }        
    } 
})

export const {login, logout} = authSlice.actions;
export default authSlice.reducer;