import {
    Alert,
    Button,
    Checkbox,
    FormControl,
    FormControlLabel,
    FormGroup,
    FormLabel,
    Paper,
    Snackbar,
    styled,
} from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { Box } from "@mui/system";
import { DataGrid } from "@mui/x-data-grid";
import axios from "axios";
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { useNavigate } from "react-router-dom";
import { oppass, opfail } from "./state/common";

import Scope from "./components/Scope";
function Tag({setToastOpen}){
    
    return (
        <Grid2 container direction={`column`} sx={{ height: "100%" }}>
            <Grid2>
                <Grid2 container>
                </Grid2>
            </Grid2>
            hello world
        </Grid2>
    )
}

export default Tag;
