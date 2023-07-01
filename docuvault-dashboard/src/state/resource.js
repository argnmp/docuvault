import { createSlice } from "@reduxjs/toolkit";

const initialState = {
    scope_ids: [],
};

export const resourceSlice = createSlice({
    name: 'resource',
    initialState,
    reducers: {
        setScopeIds: (state, action) => {
            state.scope_ids = action.payload.scope_ids;
        },
    } 
})

export const {setScopeIds} = resourceSlice.actions;
export default resourceSlice.reducer;
