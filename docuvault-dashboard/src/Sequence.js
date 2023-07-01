import styled from "@emotion/styled";
import {
    Button,
    Dialog,
    DialogActions,
    DialogContent,
    DialogContentText,
    DialogTitle,
    TextField,
    Typography,
} from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { Box } from "@mui/system";
import axios from "axios";
import { debounce } from "debounce";
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { useLoaderData, useNavigate, Link } from "react-router-dom";
import Scope from "./components/Scope";
import { opfail, oppass } from "./state/common";

export async function loader({ params, state }) {
    try {
        const res = await axios.post(
            "http://localhost:8000/resource/sequence/all",
            {
                scope_ids: state.resource.scope_ids.map((p) => p[0]),
            },
            {
                headers: {
                    Authorization: `Bearer ${state.auth.access_token}`,
                },
                withCredentials: true,
            }
        );
        return {
            isSuccess: true,
            sequences: res.data,
            msg: "sequence list fetch success",
        };
    } catch (e) {
        console.log(e);
        return { isSuccess: false, sequences: [], msg: e.response.data };
    }
}

const Header = styled(Grid2)(({ theme }) => ({
    border: "1px solid",
    width: "100%",
    borderColor: theme.common[200],
    color: theme.common[50],
    padding: 8,
    marginBottom: 8,
    backgroundColor: theme.common[500],
}));
const Row = styled(Grid2)(({ theme }) => ({
    border: "1px solid",
    borderColor: theme.common[200],
    padding: 10,
    marginBottom: 3,
    textTransform: "unset",
}));
function Sequence({ setToastOpen }) {
    const navigate = useNavigate();
    const dispatch = useDispatch();
    const access_token = useSelector((state) => state.auth.access_token);
    const { isSuccess, sequences, msg } = useLoaderData();

    let scope_ids = useSelector((state) => state.resource.scope_ids);
    let scope_state = {};
    scope_ids.forEach((element) => {
        scope_state[`${element[0]}`] = true;
    });
    const [checked, setChecked] = useState(scope_state);

    const [title, setTitle] = useState("");

    const [open, setOpen] = useState(false);
    const handleOpen = (e) => setOpen(true);
    const handleClose = (e) => setOpen(false);

    const handleSubmit = async (e) => {
        try {
            let res = await axios.post(
                "http://localhost:8000/resource/sequence/new",
                {
                    scope_ids: Object.entries(checked)
                        .filter(([key, value]) => value == true)
                        .map(([key, value]) => Number(key)),
                    title,
                },
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            handleClose(e);
            dispatch(oppass({ msg: "create sequence success" }));
            setToastOpen(true);
            navigate("/sequence");
        } catch (e) {
            dispatch(opfail({ msg: "create sequence failed" }));
            setToastOpen(true);
        }
    };
    const handleDelete = async (e) => {
        try {
            let res = await axios.post(
                "http://localhost:8000/resource/sequence/delete",
                {
                    seq_id: Number(e.target.value),
                },
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            handleClose(e);
            dispatch(oppass({ msg: "delete sequence success" }));
            setToastOpen(true);
            navigate("/sequence");
        } catch (e) {
            console.log(e);
            dispatch(opfail({ msg: e.resource.data }));
            setToastOpen(true);
        }
    };

    useEffect(() => {
        if (isSuccess) {
            dispatch(oppass({ msg }));
        } else {
            dispatch(opfail({ msg }));
            setToastOpen(true);
        }
    }, []);

    return (
        <Grid2 container direction={`column`} sx={{ height: "100%" }}>
            <Header container justifyContent={"space-between"}>
                <Grid2>sequence list</Grid2>
                <Grid2>
                    <Button
                        variant="text"
                        onClick={handleOpen}
                        sx={{
                            fontSize: 10,
                            color: "inherit",
                            border: "1px solid",
                            borderColor: "inherit",
                        }}
                    >
                        new
                    </Button>
                </Grid2>
            </Header>
            <Grid2>
                {sequences.map((elem) => {
                    return (
                        <Row
                            container
                            key={elem.id}
                            justifyContent="space-between"
                        >
                            <Button
                                sx={{ fontSize: 14, cursor: "pointer", textTransform: "unset" }}
                                onClick={(e) => {
                                    navigate(`/sequence/${elem.id}`);
                                }}
                            >
                                {elem.title}
                            </Button>
                            <Button
                                value={elem.id}
                                variant="outlined"
                                color="error"
                                onClick={handleDelete}
                                sx={{
                                    fontSize: 10,
                                }}
                            >
                                delete
                            </Button>
                        </Row>
                    );
                })}
            </Grid2>
            <Dialog open={open} onClose={handleClose}>
                <DialogTitle>Add new sequence</DialogTitle>
                <DialogContent>
                    <Scope checked={checked} setChecked={setChecked} />
                    <TextField
                        margin="dense"
                        id="title"
                        label="title"
                        fullWidth
                        variant="standard"
                        onChange={debounce((e) => {
                            console.log(e.target.value);
                            setTitle(e.target.value);
                        }, 700)}
                    />
                </DialogContent>
                <DialogActions>
                    <Button
                        sx={{ fontSize: 10 }}
                        onClick={handleClose}
                        variant={"outlined"}
                        color={"error"}
                    >
                        cancel
                    </Button>
                    <Button
                        sx={{ fontSize: 10 }}
                        onClick={handleSubmit}
                        variant={"outlined"}
                        color={"success"}
                    >
                        submit
                    </Button>
                </DialogActions>
            </Dialog>
        </Grid2>
    );
}

export default Sequence;
