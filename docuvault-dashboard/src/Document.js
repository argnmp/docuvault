import { Typography, styled, Button, Paper } from "@mui/material";
import { Box } from "@mui/system";
import axios from "axios";
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { useLoaderData } from "react-router-dom";
import { opfail, oppass } from "./state/common";

export async function loader({ params, state }) {
    try {
        const res = await axios.post(
            "http://localhost:8000/document/publish",
            {
                doc_id: Number(params.doc_id),
                scope_ids: state.resource.scope_ids.map((elem) => elem[0]),
                c_type: Number(0),
            },
            {
                headers: {
                    Authorization: `Bearer ${state.auth.access_token}`,
                },
                withCredentials: true,
            }
        );
        return { isSuccess: true, publish_token: res.data.publish_token };
    } catch (e) {
        console.log(e);
        return { isSuccess: false, publish_token: "", msg: e.response.data };
    }
}
const Wrapper = styled(Box)(({ theme }) => ({
    border: "1px solid",
    borderColor: theme.common[200],
    padding: 10,
    height: "100%",
}));
const Tag = styled(Button)(({ theme }) => ({
    boxSizing: "border-box",
    fontSize: 10,
    padding: "5px",
    marginRight: "4px",
}));
const Convert = styled(Button)(({ theme }) => ({
    boxSizing: "border-box",
    fontSize: 10,
    padding: "5px",
    marginRight: "4px",
}));

export default function Document({ setToastOpen }) {
    const access_token = useSelector((state) => state.auth.access_token);
    const dispatch = useDispatch();
    const { isSuccess, publish_token, msg } = useLoaderData();
    const [document, setDocument] = useState({});
    const [isLoaded, setIsLoaded] = useState(false);
    const tzoffset = new Date().getTimezoneOffset();
    const created_at = new Date(document.created_at);
    created_at.setSeconds(created_at.getSeconds() - tzoffset * 60);
    const updated_at = new Date(document.updated_at);
    updated_at.setSeconds(updated_at.getSeconds() - tzoffset * 60);
    useEffect(() => {
        if (isSuccess) {
            const fetchDocument = async () => {
                const res = await axios.post("http://localhost:8000/document", {
                    publish_token,
                });
                console.log(res.data);
                setDocument(res.data);
                setIsLoaded(true);
                dispatch(oppass({ msg: "document fetch success" }));
                setToastOpen(true);
            };
            fetchDocument();
        } else {
            dispatch(opfail({ msg }));
            setToastOpen(true);
        }
    }, []);

    const converts = [
        {
            id: 0,
            extension: "html",
        },{
            id: 1,
            extension: "html",
        },{
            id: 2,
            extension: "txt",
        },{
            id: 3,
            extension: "docx",
        },{
            id: 4,
            extension: "pdf",
        },{
            id: 5,
            extension: "epub",
        },{
            id: 6,
            extension: "json",
        }
    ];
    let convert_request = async (c_type) => {
        try {
            const res = await axios.post("http://localhost:8000/document/convert", 
                {
                    doc_id: document.id,
                    c_type,

                },
                {
                    headers: {
                        Authorization: `Bearer ${access_token}`,
                    },
                    withCredentials: true,
                });
            dispatch(oppass({ msg: "convert request success" }));
            setToastOpen(true);
        } catch (e) {
            dispatch(opfail({ msg: "convert request failed" }));
            setToastOpen(true);
        }
         
    }

    return (
        <Wrapper>
            {isLoaded && (
                <Box sx={{ width: "100%" }}>
                    <Typography variant="h2">{document.title}</Typography>
                    <Box sx={{ marginBottom: 1 }}>
                {converts.map((element, index) => {
                    if(index==0) return;
                    console.log(document.convert);
                    let flag = false;
                    let target;
                    for (let elem of document.convert) {
                        if(Number(elem.c_type) == index){
                            flag = true;
                            target = elem;
                        } 
                    }
                    if(flag){
                        return (
                            <Convert
                            key={index}
                            variant={`contained`}
                            color="success"
                            onClick={(e) => {
                                e.preventDefault();
                                window.location.href = `http://localhost:8000/file/${target.object_id}`;
                            }}
                            >
                            {target.extension}
                            </Convert>
                            
                        ) 
                    }
                    else {
                        return (
                            <Convert
                                key={index}
                            variant={`contained`}
                            color = "error"

                            onClick={(e)=>{
                                e.preventDefault();
                                convert_request(index)              
                            }}>
                                {element.extension}
                            </Convert>
                        )
                    }
                })}
                    </Box>
                    {document.tags.map((element) => {
                        return (
                            <Tag key={`${element.id}`} variant={`contained`}>
                                {element.value}
                            </Tag>
                        );
                    })}
                    <Typography variant="subtitle2" align="right">
                        created at: {created_at.toString()}
                    </Typography>
                    <Typography variant="subtitle2" align="right">
                        updated at: {updated_at.toString()}
                    </Typography>
                    <Typography variant="subtitle2" align="right">
                        status:{document.status}
                    </Typography>
                    <Box
                        sx={{ paddingLeft: "20px" }}
                        dangerouslySetInnerHTML={{ __html: document.data }}
                    ></Box>
                </Box>
            )}
        </Wrapper>
    );
}
