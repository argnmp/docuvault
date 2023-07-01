import { Alert, AlertTitle } from "@mui/material";
import Grid2 from "@mui/material/Unstable_Grid2/Grid2";

function Error(){
    return (
        <Alert severity="error">
            <AlertTitle>Error</AlertTitle>
            404 Not found
        </Alert>
    )
}
export default Error;