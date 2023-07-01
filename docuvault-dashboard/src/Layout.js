import { Link, Navigate, useNavigate } from "react-router-dom";
import {
    Alert,
    Button,
    CssBaseline,
    Divider,
    Grid,
    List,
    ListItem,
    ListItemButton,
    ListItemIcon,
    ListItemText,
    MenuItem,
    MenuList,
    Paper,
    Snackbar,
    styled,
    Typography,
} from "@mui/material";
import LoginIcon from "@mui/icons-material/Login";
import ListAltIcon from "@mui/icons-material/ListAlt";
import AddIcon from "@mui/icons-material/Add";
import { Box, Container } from "@mui/system";

import { createTheme, ThemeProvider } from "@mui/material/styles";
import {
    blue,
    blueGrey,
    grey,
    indigo,
    lightBlue,
    red,
    teal,
} from "@mui/material/colors";
import { useDispatch, useSelector } from "react-redux";
import { logout } from "./state/auth";
import React, { useState } from "react";

const theme = createTheme({
    highlight: indigo,
    common: blueGrey,
});

const Logo = styled(Box)(({ theme }) => ({
    marginBottom: "8px",
    boxSizing: "border-box",
    width: "100%",
    border: "1px solid",
    borderColor: theme.highlight[200],
    color: theme.highlight[900],
    backgroundColor: theme.highlight[50],
}));

const Content = styled(Box)(({ theme }) => ({
    boxSizing: "border-box",
    width: "100%",
    border: "1px solid",
    borderColor: theme.highlight[200],
    color: theme.highlight[900],
    backgroundColor: theme.highlight[50],
}));

function Layout({ children }) {
    const dispatch = useDispatch();
    const navigate = useNavigate();
    const isLogined = useSelector((state) => state.auth.isLogined);

    const [toastOpen, setToastOpen] = useState(false);
    const isSucceeded = useSelector((state) => state.common.isSucceeded);
    const msg = useSelector((state) => state.common.msg);

    const childrenwithprop = React.Children.map(children, child => {
        if (React.isValidElement(child)) {
            return React.cloneElement(child, { setToastOpen }, null);
        }
        return child;
    });

    const handleClose = (event, reason) => {
        if (reason === "clickaway") {
            return;
        }
        setToastOpen(false);
    };
    return (
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <Grid container sx={{ height: "100%", padding: 1 }}>
                <Grid
                    item
                    xs={3}
                    md={2}
                    borderColor={"primary.main"}
                    sx={{ height: "100%" }}
                >
                    <Logo>
                        <Button
                            sx={{ width: "100%" }}
                            color="inherit"
                            onClick={() => navigate("/")}
                        >
                            docuvault
                        </Button>
                    </Logo>
                    <Content>
                        <List component="nav" dense>
                            <ListItem disablePadding>
                                {isLogined ? (
                                    <ListItemButton
                                        onClick={() => {
                                            dispatch(logout());
                                            navigate("/");
                                        }}
                                    >
                                        <ListItemIcon>
                                            <LoginIcon fontSize="small" />
                                        </ListItemIcon>
                                        <ListItemText>logout</ListItemText>
                                    </ListItemButton>
                                ) : (
                                    <ListItemButton
                                        onClick={() => navigate("/login")}
                                    >
                                        <ListItemIcon>
                                            <LoginIcon fontSize="small" />
                                        </ListItemIcon>
                                        <ListItemText>login</ListItemText>
                                    </ListItemButton>
                                )}
                            </ListItem>
                            <ListItem disablePadding>
                                <ListItemButton
                                    onClick={() => navigate("/write")}
                                >
                                    <ListItemIcon>
                                        <AddIcon fontSize="small" />
                                    </ListItemIcon>
                                    <ListItemText>write</ListItemText>
                                </ListItemButton>
                            </ListItem>
                            <ListItem disablePadding>
                                <ListItemButton
                                    onClick={() => navigate("/list")}
                                >
                                    <ListItemIcon>
                                        <ListAltIcon fontSize="small" />
                                    </ListItemIcon>
                                    <ListItemText>list</ListItemText>
                                </ListItemButton>
                            </ListItem>
                            <ListItem disablePadding>
                                <ListItemButton
                                    onClick={() => navigate("/sequence")}
                                >
                                    <ListItemIcon>
                                        <ListAltIcon fontSize="small" />
                                    </ListItemIcon>
                                    <ListItemText>sequence</ListItemText>
                                </ListItemButton>
                            </ListItem>
                        </List>
                    </Content>
                </Grid>
                <Grid item xs={9} md={10} sx={{ paddingLeft: 1 }}>
                    {childrenwithprop}
                </Grid>
            </Grid>

            <Snackbar
                open={toastOpen}
                autoHideDuration={5000}
                onClose={handleClose}
            >
                {isSucceeded ? (
                    <Alert onClose={handleClose} severity={"success"}>
                        {msg}
                    </Alert>
                ) : (
                    <Alert onClose={handleClose} severity={"error"}>
                        {msg}
                    </Alert>
                )}
            </Snackbar>
        </ThemeProvider>
    );
}
export default Layout;
