import { createSlice } from "@reduxjs/toolkit";

const initialState = {
    isSucceeded: true,
    msg: "", 
};

export const commonSlice = createSlice({
    name: 'common',
    initialState,
    reducers: {
        oppass: (state, action) => {
            state.isSucceeded = true;
            state.msg = action.payload.msg;
        },
        opfail: (state, action) => {
            state.isSucceeded = false;
            state.msg = action.payload.msg;
        }        
    } 
})

export const {oppass, opfail} = commonSlice.actions;
export default commonSlice.reducer;
