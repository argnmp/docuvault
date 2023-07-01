import {
    Button,
    Container,
    FormControl,
    Grid,
    Input,
    TextField,
} from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2";
import { useState } from "react";
import axios from "axios";
import { Navigate, useNavigate } from "react-router-dom";
import { useDispatch, useSelector } from "react-redux";
import { login } from "./state/auth";
import { saveState } from "./state";
import {oppass, opfail} from "./state/common";
import {setScopeIds} from "./state/resource";

function Login({setToastOpen}) {
    const dispatch = useDispatch();
    const navigate = useNavigate();
    const [email, setEmail] = useState("");
    const [pw, setPw] = useState("");
    let isLogined = useSelector((state) => state.auth.isLogined);

    const handleLogin = async (e) => {
        e.preventDefault();
        try {
            let res = await axios.post(
                "http://localhost:8000/auth/issue",
                {
                    email,
                    password: pw,
                },
                {
                    headers: {
                        "Content-type": "application/json",
                    },
                }
            );
            saveState();
            dispatch(login({ access_token: res.data.access_token }));

            res = await axios.post(
                "http://localhost:8000/resource/scope/all",
                {},
                {
                    headers: {
                        Authorization: `Bearer ${res.data.access_token}`,
                    },
                    withCredentials: true,
                }
            );
            let scope_ids = res.data.scopes;
            console.log(scope_ids);
            dispatch(setScopeIds({scope_ids: scope_ids}));

            dispatch(oppass({msg: "login success"}));
            setToastOpen(true);
            navigate("/list");
        } catch (e) {
            dispatch(opfail({msg: `${e.response.data}`}));
            setToastOpen(true);
            console.log(e);
            return;
        }
    };
    return (
        <div className="App">
            <Grid2 container justifyContent={"center"} alignItems={"center"}>
                <Grid2>
                    <form onSubmit={async (e) => handleLogin(e)}>
                        <FormControl>
                            <TextField
                                id="filled-basic"
                                label="email"
                                variant="filled"
                                margin="dense"
                                size="small"
                                onChange={(e) => setEmail(e.target.value)}
                            />
                            <TextField
                                id="filled-basic"
                                label="password"
                                variant="filled"
                                margin="dense"
                                size="small"
                                type={"password"}
                                onChange={(e) => setPw(e.target.value)}
                            />
                            <Button
                                type="submit"
                                variant="contained"
                                sx={{ marginTop: 1 }}
                            >
                                login
                            </Button>
                        </FormControl>
                    </form>
                </Grid2>
            </Grid2>
        </div>
    );
}

export default Login;
