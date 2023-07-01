import {
    Checkbox,
    FormControl,
    FormControlLabel,
    FormGroup,
    FormLabel,
} from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";
import {Box} from "@mui/system";
import {useSelector} from "react-redux";

function Scope({checked, setChecked}) {
    let scope_ids = useSelector((state)=>state.resource.scope_ids);

    const handleChange = (event) => {
        let new_state = {
            ...checked,
            [event.target.name]: event.target.checked,
        };
        setChecked(new_state);
    };
    return (
        <Grid2>
            <FormControl component="fieldset">
                <FormGroup row>
                    {scope_ids.map((elem) => {
                        return (
                            <FormControlLabel
                                key={`${elem[0]}`}
                                control={
                                    <Checkbox
                                        size={`small`}
                                        checked={checked[`${elem[0]}`]}
                                        onChange={handleChange}
                                        name={`${elem[0]}`}
                                    />
                                }
                                label={<Box sx={{fontSize: 14}}>{`${elem[1]}`}</Box>}
                            />
                        );
                    })}
                </FormGroup>
            </FormControl>
        </Grid2>
    );
}
export default Scope;
