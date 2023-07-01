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

const Header = styled(Box)(({ theme }) => ({
    boxSizing: "border-box",
    width: "100%",
    border: "1px solid",
    borderColor: theme.common[200],
    padding: 8,
    marginBottom: 8,
    color: theme.common[900],
    backgroundColor: theme.common[50],
}));
const Tag = styled(Button)(({ theme }) => ({
    boxSizing: "border-box",
    fontSize: 10,
    padding: "5px",
    marginRight: "4px",
}));

function List({ setToastOpen }) {
    const dispatch = useDispatch();
    const navigate = useNavigate();

    let [list, setList] = useState([]);
    let [tags, setTags] = useState([]);
    let [selectedState, setSelectedState] = useState([]);

    let scope_ids = useSelector((state) => state.resource.scope_ids);
    let scope_state = {};
    scope_ids.forEach((element) => {
        scope_state[`${element[0]}`] = true;
    });
    let [checked, setChecked] = useState(scope_state);
    const [activated, setActivated] = useState(false);

    let access_token = useSelector((state) => state.auth.access_token);

    const tzoffset = new Date().getTimezoneOffset();

    const handleChange = (event) => {
        let new_state = {
            ...checked,
            [event.target.name]: event.target.checked,
        };
        setChecked(new_state);
    };
    const handleRefresh = async (event) => {
        await get_tag_list();
        await get_document_list();
    };
    const handleDelete = async (event) => {
        if (selectedState.length == 0) return;
        try {
            let res = await axios.post(
                "http://localhost:8000/document/delete",
                {
                    doc_ids: selectedState,
                },
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            dispatch(oppass({ msg: "selected documents are deleted" }));
            handleRefresh();
            setToastOpen(true);
        } catch (e) {
            console.log(e);
            dispatch(opfail({ msg: "document deletion failed" }));
            setToastOpen(true);
        }
    };

    const handleClose = (event, reason) => {
        if (reason === "clickaway") {
            return;
        }
        setToastOpen(false);
    };
    const get_tag_list = async () => {
        try {
            let req = Object.entries(checked)
                .filter(([key, value]) => value == true)
                .map(([key, value]) => Number(key));
            let res = await axios.post(
                "http://localhost:8000/resource/tag",
                {
                    scope_ids: req,
                },
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            let tmptags = {};
            res.data.forEach((elem) => {
                tmptags[`${elem.id}`] = {
                    name: elem.value,
                    status: true,
                };
            });
            setTags(tmptags);
            console.log(res.data);
            dispatch(oppass({ msg: "document list fetch success" }));
            setToastOpen(true);
        } catch (e) {
            dispatch(opfail({ msg: `${e.response.data}` }));
            setToastOpen(true);
        }
    };
    const get_document_list = async () => {
        try {
            let req = Object.entries(checked)
                .filter(([key, value]) => value == true)
                .map(([key, value]) => Number(key));

            let tag_ids = Object.entries(tags)
                .filter(([key, value]) => value.status == true)
                .map(([key, value]) => Number(key));

            let body = {
                scope_ids: req,
            };
            if(tag_ids.length==1){
                body[`tag_id`] = tag_ids[0];
            }

            let res = await axios.post(
                "http://localhost:8000/resource/list",
                body
                ,
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );

            setList(res.data);
            dispatch(oppass({ msg: "document list fetch success" }));
            setToastOpen(true);
        } catch (e) {
            dispatch(opfail({ msg: `${e.response.data}` }));
            setToastOpen(true);
        }
    };

    useEffect(() => {
        get_tag_list();
        get_document_list();
    }, []);

    const columns = [
        { field: "id", headerName: "id", width: 20 },
        { field: "title", headerName: "title", width: 250 },
        {
            field: "created_at",
            headerName: "created_at",
            width: 200,
            valueFormatter: (params) => {
                const created_at = new Date(params.value);
                created_at.setSeconds(created_at.getSeconds() - tzoffset * 60);
                return created_at.toLocaleString();
            },
        },
        {
            field: "updated_at",
            headerName: "updated_at",
            width: 200,
            valueFormatter: (params) => {
                const updated_at = new Date(params.value);
                updated_at.setSeconds(updated_at.getSeconds() - tzoffset * 60);
                return updated_at.toLocaleString();
            },
        },
        { field: "scope_ids", headerName: "scopes", width: 100 },
        { field: "seq_ids", headerName: "sequences", width: 100 },
        {
            field: "openbtn",
            headerName: "-",
            width: 80,
            renderCell: (params) => {
                return (
                    <Button
                        sx={{ width: "100%", fontSize: "10px" }}
                        variant="outlined"
                        onClick={(e) => {
                            navigate(`/document/${params.id}`);
                        }}
                        size="small"
                    >
                        open
                    </Button>
                );
            },
        },
        {
            field: "updatebtn",
            headerName: "-",
            width: 80,
            renderCell: (params) => {
                return (
                    <Button
                        sx={{ width: "100%", fontSize: "10px" }}
                        variant="outlined"
                        color="warning"
                        onClick={(e) => {
                            navigate(`/update/${params.id}`);
                        }}
                        size="small"
                    >
                        update
                    </Button>
                );
            },
        },
    ];
    return (
        <Grid2 container direction={`column`} sx={{ height: "100%" }}>
            <Grid2>
                <Grid2 container>
                    <Header>
                        <Scope checked={checked} setChecked={setChecked} />
                    </Header>
                    <Header>
                        {Object.entries(tags).map(([key, value]) => {
                            return (
                                <Tag
                                    key={`${key}`}
                                    variant={`contained`}
                                    color={`${value.status ? "info" : "grey"}`}
                                    value={key}
                                    onClick={(e) => {
                                        let tmptags = {};
                                        if (!tags[e.target.value].status) {
                                            Object.entries(tags).forEach(
                                                ([key, value]) => {
                                                    tmptags[`${key}`] = {
                                                        name: value.name,
                                                        status: true,
                                                    };
                                                }
                                            );
                                        } else {
                                            Object.entries(tags).forEach(
                                                ([key, value]) => {
                                                    if (key == e.target.value) {
                                                        tmptags[`${key}`] = {
                                                            name: value.name,
                                                            status: true,
                                                        };
                                                    } else {
                                                        tmptags[`${key}`] = {
                                                            name: value.name,
                                                            status: false,
                                                        };
                                                    }
                                                }
                                            );
                                        }
                                        setTags(tmptags);
                                    }}
                                >
                                    {value.name}
                                </Tag>
                            );
                        })}
                    </Header>
                </Grid2>
            </Grid2>
            <Grid2 xs={12}>
                <Button
                    variant="contained"
                    onClick={(e) => handleRefresh()}
                    sx={{
                        width: "100%",
                        fontSize: "10px",
                        padding: "5px",
                        marginBottom: "5px",
                    }}
                >
                    refresh
                </Button>
            </Grid2>
            <Grid2 sx={{ flexGrow: 1 }}>
                <DataGrid
                    sx={{ height: "100%", borderRadius: 0 }}
                    columns={columns}
                    rows={list}
                    checkboxSelection
                    disableSelectionOnClick
                    onSelectionModelChange={(ids) => {
                        if (ids.length > 0) {
                            setActivated(true);
                        } else if (ids.length == 0) {
                            setActivated(false);
                        }

                        setSelectedState(ids);
                    }}
                />
            </Grid2>
            <Grid2 xs={12}>
                <Button
                    variant="contained"
                    onClick={(e) => handleDelete()}
                    sx={{ width: "100%", fontSize: "10px", padding: "5px" }}
                    color="error"
                    disabled={!activated}
                >
                    delete
                </Button>
            </Grid2>
        </Grid2>
    );
}
export default List;
