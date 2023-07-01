import { FormControl, InputLabel, MenuItem, Select } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import axios from "axios";
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { opfail, oppass } from "../state/common";

function SequenceSelect({ sequenceId, setSequenceId, setToastOpen }) {
    const dispatch = useDispatch();
    const scope_ids = useSelector((state) => state.resource.scope_ids);
    const access_token = useSelector((state) => state.auth.access_token);
    const [sequences, setSequences] = useState([]);
    useEffect(() => {
        const get_sequences = async () => {
            try {
                const res = await axios.post(
                    "http://localhost:8000/resource/sequence/all",
                    {
                        // because this is dashboard, request with the full scope
                        scope_ids: scope_ids.map((p) => p[0]),
                    },
                    {
                        headers: {
                            Authorization: `Bearer ${access_token}`,
                        },
                        withCredentials: true,
                    }
                );
                setSequences(res.data);

                dispatch(oppass({ msg: "sequences fetch success" }));
                setToastOpen(true);
            } catch (e) {
                console.log(e);
                dispatch(opfail({ msg: e.responase.data }));
                setToastOpen(true);
            }
        };
        get_sequences();
    }, []);
    return (
        <Grid2>
            <FormControl>
            <InputLabel id="sequences">sequence</InputLabel>
                <Select
                    id="sequences"
                    labelId="sequences"
                    value={sequenceId}
                    label="sequence"
                    sx={{fontSize: 12, width: 100}}
                    onChange={(e) => {
                        setSequenceId(e.target.value);
                    }}
                    size="small"
                >
                    <MenuItem sx={{fontSize: 12}} key={0} value={0}>
                        none
                    </MenuItem>
                    {sequences.map((elem) => {
                        return (
                            <MenuItem sx={{fontSize: 12}} key={elem.id} value={elem.id}>
                                {elem.title}
                            </MenuItem>
                        );
                    })}
                </Select>
            </FormControl>
        </Grid2>
    );
}
export default SequenceSelect;
