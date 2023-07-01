import { Button } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { Box } from "@mui/system";
import { DataGrid } from "@mui/x-data-grid";
import axios from "axios";
import { useEffect, useRef, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { useLoaderData, useNavigate } from "react-router-dom";
import { opfail, oppass } from "./state/common";

export async function loader({ params, state }) {
    return { seq_id: params.seq_id };
}

function SequenceList({ setToastOpen }) {
    const dispatch = useDispatch();
    const navigate = useNavigate();
    const { seq_id } = useLoaderData();
    let access_token = useSelector((state) => state.auth.access_token);
    let scope_ids = useSelector((state) => state.resource.scope_ids);
    scope_ids = scope_ids.map((elem) => Number(elem[0]));

    const [activated, setActivated] = useState(false);
    const [list, setList] = useState([]);
    const listRef = useRef(list);

    useEffect(() => {
        const get_sequence_list = async () => {
            try {
                let res = await axios.post(
                    "http://localhost:8000/resource/sequence/list",
                    {
                        scope_ids,
                        seq_id: Number(seq_id),
                    },
                    {
                        headers: {
                            Authorization: `Bearer ${access_token}`,
                        },
                        withCredentials: true,
                    }
                );
                listRef.current = res.data;
                setList(res.data);
                dispatch(oppass({ msg: "document list fetch success" }));
                setToastOpen(true);
            } catch (e) {
                dispatch(opfail({ msg: `${e.response.data}` }));
                setToastOpen(true);
            }
        };
        get_sequence_list();
    }, []);
    const handleSequenceUpdate = async (e) => {
        e.preventDefault();
        try {
            let body = {
                seq_id: Number(seq_id),
                order: list.map((elem, idx) => {
                    return {
                        doc_id: Number(elem.id),
                        seq_order: idx + 1,
                    };
                }),
            };
            let res = await axios.post(
                "http://localhost:8000/resource/sequence/update",
                body,
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            dispatch(oppass({ msg: "sequence update success" }));
            setToastOpen(true);
            navigate(0);
        } catch (e) {
            console.log(e);
            dispatch(opfail({ msg: e.resource.data }));
            setToastOpen(true);
        }
    };
    const tzoffset = new Date().getTimezoneOffset();
    const columns = [
        { field: "id", headerName: "id", width: 20, sortable: false },
        { field: "seq_order", headerName: "order", width: 60, sortable: false },
        { field: "title", headerName: "title", width: 200, sortable: false },
        {
            field: "created_at",
            headerName: "created_at",
            width: 200,
            valueFormatter: (params) => {
                const created_at = new Date(params.value);
                created_at.setSeconds(created_at.getSeconds() - tzoffset * 60);
                return created_at.toLocaleString();
            },
            sortable: false,
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
            sortable: false,
        },
        {
            field: "scope_ids",
            headerName: "scopes",
            width: 100,
            sortable: false,
        },
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
            sortable: false,
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
            sortable: false,
        },
        {
            field: "upbtn",
            headerName: "-",
            width: 120,
            renderCell: (params) => {
                return (
                    <Box sx={{ width: "100%" }}>
                        <Button
                            sx={{ minWidth: "40px", fontSize: "10px" }}
                            variant="outlined"
                            color="success"
                            onClick={(e) => {
                                let templist = listRef.current.slice();
                                let cur_idx = templist.findIndex(
                                    (e) => e.id == params.id
                                );
                                if (cur_idx != 0) {
                                    templist[cur_idx - 1] = templist.splice(
                                        cur_idx,
                                        1,
                                        templist[cur_idx - 1]
                                    )[0];
                                }
                                listRef.current = templist;
                                setList(templist);
                                if (!activated) setActivated(true);
                            }}
                            size="small"
                        >
                            up
                        </Button>
                        <Button
                            sx={{ minWidth: "40px", fontSize: "10px" }}
                            variant="outlined"
                            color="success"
                            onClick={(e) => {
                                let templist = listRef.current.slice();
                                let cur_idx = templist.findIndex(
                                    (e) => e.id == params.id
                                );
                                if (cur_idx != templist.length - 1) {
                                    templist[cur_idx] = templist.splice(
                                        cur_idx + 1,
                                        1,
                                        templist[cur_idx]
                                    )[0];
                                }
                                listRef.current = templist;
                                setList(templist);
                                if (!activated) setActivated(true);
                            }}
                            size="small"
                        >
                            down
                        </Button>
                    </Box>
                );
            },
            sortable: false,
        },
    ];
    return (
        <Grid2 container direction={`column`} sx={{ height: "100%" }}>
            <Grid2>
                <Button
                    variant="contained"
                    onClick={(e) => handleSequenceUpdate(e)}
                    sx={{ width: '100%', fontSize: "10px", padding: "5px" }}
                    color="warning"
                    disabled={!activated}
                >
                    sequence_update
                </Button>
            </Grid2>
            <Grid2 sx={{ flexGrow: 1 }}>
                <DataGrid
                    sx={{ height: "100%", borderRadius: 0 }}
                    columns={columns}
                    rows={list}
                    checkboxSelection
                    disableSelectionOnClick
                    disableColumnMenu={true}
                />
            </Grid2>
        </Grid2>
    );
}

export default SequenceList;
