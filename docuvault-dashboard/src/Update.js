import styled from "@emotion/styled";
import {
    Button,
    TextField,
    Checkbox,
    FormControl,
    FormControlLabel,
    FormGroup,
    FormLabel,
    Divider,
} from "@mui/material";

import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import { Box } from "@mui/system";
import axios from "axios";
import { useEffect, useRef, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { opfail, oppass } from "./state/common";
import { useLoaderData, useNavigate } from "react-router-dom";
import { debounce } from "debounce";
import Scope from "./components/Scope";
import SequenceSelect from "./components/SequenceSelect";

export async function loader({ params, state }) {
    try {
        const res = await axios.post(
            `http://localhost:8000/document/get_update_resource/${params.doc_id}`,
            {},
            {
                headers: {
                    Authorization: `Bearer ${state.auth.access_token}`,
                },
                withCredentials: true,
            }
        );
        return { update_resource: res.data };
    } catch (e) {
        console.log(e);
        return;
    }
}

const Wrapper = styled(Box)(({ theme }) => ({
    border: "1px solid",
    borderColor: theme.common[200],
    padding: 10,
    height: "100%",
}));
const Btn = styled(Button)(({ theme }) => ({
    boxSizing: "border-box",
    fontSize: 10,
    padding: "5px",
    marginRight: "4px",
}));

const TextArea = styled(TextField)(({ theme }) => ({
    width: "100%",
    height: "100%",
}));
const Tag = styled(Button)(({ theme }) => ({
    boxSizing: "border-box",
    fontSize: 10,
    padding: "5px",
    marginRight: "4px",
    marginBottom: "4px",
}));
export default function Update({ setToastOpen }) {
    const dispatch = useDispatch();
    const navigate = useNavigate();
    const { update_resource } = useLoaderData();

    const [raw, setRaw] = useState(update_resource.raw);

    let tmp_tag_id = useRef({
        cursor: 1,
    });
    const [refresh, setRefresh] = useState(true);
    const flip = ()=>setRefresh(!refresh);

    let inital_tags = update_resource.tags.map((tag, index)=>({id: index, value: tag}));
    const [tags, setTags] = useState(inital_tags);
    let access_token = useSelector((state) => state.auth.access_token);

    let scope_ids = useSelector((state) => state.resource.scope_ids);

    let scope_state = {};
    scope_ids.forEach((element) => {
        scope_state[`${element[0]}`] = update_resource.scope_ids.findIndex((elem)=>elem==element[0]) == -1 ? false : true;
    });
    let [checked, setChecked] = useState(scope_state);
    const [sequenceId, setSequenceId] = useState(update_resource.seq_ids.length > 0 ? update_resource.seq_ids[0] : 0);

    console.log(update_resource);

    const handleSubmit = async (e) => {
        e.preventDefault();
        if (raw == "") return;
        try {
            let updated_scope_ids = Object.entries(checked)
                        .filter(([key, value]) => value == true)
                        .map(([key, value]) => Number(key));
            let body = {
                doc_id: update_resource.id,
                raw,
                tags: tags.map(({ id, value }) => value),
                scope_ids: updated_scope_ids,
            };
            if(sequenceId != 0){
                body[`seq_id`] = sequenceId;

            }
            let res = await axios.post(
                "http://localhost:8000/document/update",
                body,
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                }
            );
            dispatch(oppass({ msg: "update success" }));
            navigate(`/document/${update_resource.id}`);
            setToastOpen(true);
        } catch (e) {
            dispatch(opfail({ msg: e.response.data }));
            setToastOpen(true);
            return;
        }
    };
    const handleChange = (event) => {
        let new_state = {
            ...checked,
            [event.target.name]: event.target.checked,
        };
        setChecked(new_state);
    };
    const handleTagAdd = (e) => {
        if (e.target.value != "") {
            setTags([
                ...tags,
                { id: tmp_tag_id.current.cursor, value: e.target.value },
            ]);
            tmp_tag_id.current.cursor += 1;
            e.target.value = "";
        }
    };
    const handleTagDelete = (e) => {
        e.preventDefault();
        const tag_id = e.target.getAttribute("tag_id");
        setTags(tags.filter((elem) => elem.id != tag_id));
    };
    let mimereg = RegExp("(^image)(\/)[a-zA-Z0-9_]*");
    const handleUpload = async (e) => {
        e.preventDefault();
        try {
            let formData = new FormData();
            for(let file of e.target.files){
                formData.append("files", file);
            }
            let res = await axios.post(
                "http://localhost:8000/file/upload",
                formData,
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                        "Content-Type": "multipart/form-data",
                    },
                    withCredentials: true,
                }
            );
            console.log(res.data);
            let tempraw = raw;
            for(let file of res.data){
                if(mimereg.test(file.ftype)){
                    tempraw += `\n![${file.name}](http://localhost:8000/file/${file.object_id})`;
                }
                else {
                    tempraw += `\n[${file.name}](http://localhost:8000/file/${file.object_id})`;
                }
            }
            tempraw += "\n";
            setRaw(tempraw);
            dispatch(oppass({ msg: "upload success" }));
            setToastOpen(true);
        } catch(e) {
            console.log(e);
            dispatch(opfail({ msg: e.response.data }));
            setToastOpen(true);
        }

    }
    return (
        <Wrapper>
            <Grid2 container justifyContent={"space-between"}>
                <Grid2 container>
                    <Grid2>
                        <Scope checked={checked} setChecked={setChecked} />
                    </Grid2>
                    <SequenceSelect
                        sequenceId={sequenceId}
                        setSequenceId={setSequenceId}
                        setToastOpen={setToastOpen}
                    />
                </Grid2>
                <Grid2>
                    <FormControl>
                        <Box>
                            <input id="file-input" type="file" multiple onChange={(e)=>handleUpload(e)}/>
                        </Box>
                    </FormControl>
                </Grid2>
            </Grid2>

            <Grid2 container>
                <Grid2 sm={12} sx={{ marginBottom: "8px" }}>
                    <TextArea
                        multiline
                        rows={20}
                        value={raw}
                        onChange={(e) => {
                            setRaw(e.target.value);
                        }}
                    />
                </Grid2>
                <Grid2 item xs={10}>
                    {tags.map((elem) => (
                        <Tag
                            variant="contained"
                            color="warning"
                            key={elem.id}
                            tag_id={elem.id}
                            onClick={(e) => handleTagDelete(e)}
                        >
                            {elem.value}
                        </Tag>
                    ))}

                    <TextField
                        size="small"
                        inputProps={{
                            style: {
                                padding: 5,
                                height: "18.5px",
                                width: "64px",
                                fontSize: "12px",
                            },
                        }}
                        placeholder="tag"
                        onKeyUp={(e) => {
                            if (e.keyCode == 13) {
                                e.preventDefault();
                                handleTagAdd(e);
                            }
                        }}
                    />
                </Grid2>
                <Grid2 sx={{ marginTop: "8px", marginBottom: "8px" }} sm={12}>
                    <Btn
                        type="submit"
                        variant="contained"
                        onClick={(e) => handleSubmit(e)}
                    >
                        submit
                    </Btn>
                </Grid2>
            </Grid2>
        </Wrapper>
    );
}
